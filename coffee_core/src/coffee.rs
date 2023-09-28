//! Coffee mod implementation

use std::collections::HashMap;
use std::fmt::Debug;
use std::vec::Vec;
use tokio::fs;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use clightningrpc_common::client::Client;
use clightningrpc_common::json_utils;
use clightningrpc_conf::{CLNConf, SyncCLNConf};
use log;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use coffee_github::repository::Github;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::repository::Repository;
use coffee_lib::types::response::*;
use coffee_lib::url::URL;
use coffee_lib::{commit_id, error, get_repo_info, sh};
use coffee_storage::model::repository::{Kind, Repository as RepositoryInfo};
use coffee_storage::nosql_db::NoSQlStorage;
use coffee_storage::storage::StorageManager;

use super::config;
use crate::config::CoffeeConf;
use crate::nurse::chain::RecoveryChainOfResponsibility;
use crate::CoffeeArgs;

pub type RepoName = String;

#[derive(Serialize, Deserialize)]
/// FIXME: move the list of plugin
/// and the list of repository inside this struct.
pub struct CoffeeStorageInfo {
    pub config: config::CoffeeConf,
    pub repositories: HashMap<RepoName, RepositoryInfo>,
}

impl From<&CoffeeManager> for CoffeeStorageInfo {
    fn from(value: &CoffeeManager) -> Self {
        let mut repos = HashMap::new();
        // FIXME: improve the down cast
        for (name, repo) in value.repos.iter() {
            let repo = if let Some(git) = repo.as_any().downcast_ref::<Github>() {
                RepositoryInfo::from(git)
            } else {
                panic!("this should never happens")
            };
            repos.insert(name.to_string(), repo);
        }

        CoffeeStorageInfo {
            config: value.config.to_owned(),
            repositories: repos, // FIXME: find a way to downcast
        }
    }
}

pub struct CoffeeManager {
    pub config: config::CoffeeConf,
    pub repos: HashMap<String, Box<dyn Repository + Send + Sync>>,
    /// Core lightning configuration managed by coffee
    pub coffee_cln_config: CLNConf,
    /// Core lightning configuration that include the
    /// configuration managed by coffee
    pub cln_config: Option<CLNConf>,
    /// storage instance to make all the plugin manager
    /// information persistent on disk
    pub storage: NoSQlStorage,
    /// core lightning rpc connection
    pub rpc: Option<Client>,
    /// Recovery Strategies for the nurse command.
    pub recovery_strategies: RecoveryChainOfResponsibility,
}

impl CoffeeManager {
    pub async fn new(conf: &dyn CoffeeArgs) -> Result<Self, CoffeeError> {
        let conf = CoffeeConf::new(conf).await?;
        let mut coffee = CoffeeManager {
            config: conf.clone(),
            coffee_cln_config: CLNConf::new(conf.config_path, true),
            repos: HashMap::new(),
            storage: NoSQlStorage::new(&conf.root_path).await?,
            cln_config: None,
            rpc: None,
            recovery_strategies: RecoveryChainOfResponsibility::new().await?,
        };
        coffee.inventory().await?;
        Ok(coffee)
    }

    /// when coffee is configured, run an inventory to collect all the necessary information
    /// about the coffee ecosystem.
    async fn inventory(&mut self) -> Result<(), CoffeeError> {
        let _ = self
            .storage
            .load::<CoffeeStorageInfo>(&self.config.network)
            .await
            .map(|store| {
                self.config = store.config;
            });
        // FIXME: check if this exist in a better wai
        let _ = self
            .storage
            .load::<HashMap<RepoName, RepositoryInfo>>("repositories")
            .await
            .map(|item| {
                log::debug!("repositories in store {:?}", item);
                item.iter().for_each(|repo| match repo.1.kind {
                    Kind::Git => {
                        let repo = Github::from(repo.1);
                        self.repos.insert(repo.name(), Box::new(repo));
                    }
                });
            });

        if let Err(err) = self.coffee_cln_config.parse() {
            log::error!("{}", err.cause);
        }
        self.load_cln_conf().await?;
        log::debug!("cln conf {:?}", self.coffee_cln_config);
        log::debug!("finish plugin manager inventory");
        Ok(())
    }

    pub async fn cln<T: Serialize, U: DeserializeOwned + Debug>(
        &self,
        method: &str,
        payload: T,
    ) -> Result<U, CoffeeError> {
        if let Some(rpc) = &self.rpc {
            let response = rpc
                .send_request(method, payload)
                .map_err(|err| error!("{}", &format!("{err}")))?;
            log::debug!("cln answer with {:?}", response);
            if let Some(err) = response.error {
                return Err(error!("{}", &format!("cln error: {}", err.message)));
            }
            return Ok(response.result.unwrap());
        }
        Err(error!("rpc connection to core lightning not available"))
    }

    pub async fn start_plugin(&self, path: &str) -> Result<(), CoffeeError> {
        let mut payload = json_utils::init_payload();
        json_utils::add_str(&mut payload, "subcommand", "start");
        json_utils::add_str(&mut payload, "plugin", path);
        let response = self
            .cln::<serde_json::Value, serde_json::Value>("plugin", payload)
            .await?;
        log::debug!("plugin registered: {response}");
        Ok(())
    }

    pub fn storage_info(&self) -> CoffeeStorageInfo {
        CoffeeStorageInfo::from(self)
    }

    pub async fn flush(&self) -> Result<(), CoffeeError> {
        let store_info = self.storage_info();
        self.storage
            .store(&self.config.network, &store_info)
            .await?;
        self.storage
            .store("repositories", &store_info.repositories)
            .await?;
        Ok(())
    }

    pub async fn update_conf(&self) -> Result<(), CoffeeError> {
        self.coffee_cln_config.flush()?;
        log::debug!("stored all the cln info in {}", self.coffee_cln_config);
        Ok(())
    }

    pub async fn load_cln_conf(&mut self) -> Result<(), CoffeeError> {
        if self.config.cln_config_path.is_none() {
            return Ok(());
        }
        let root = self.config.cln_root.clone().unwrap();
        let rpc = Client::new(format!("{root}/{}/lightning-rpc", self.config.network));
        self.rpc = Some(rpc);
        let path = self.config.cln_config_path.clone().unwrap();
        let mut file = CLNConf::new(path.clone(), true);
        log::info!("looking for the cln config: {path}");
        file.parse()
            .map_err(|err| CoffeeError::new(err.core, &err.cause))?;
        log::trace!("{:?}", file.fields);
        self.cln_config = Some(file);
        Ok(())
    }

    pub async fn setup_with_cln(&mut self, cln_dir: &str) -> Result<(), CoffeeError> {
        if self.cln_config.is_some() {
            log::warn!("you are overriding the previous set up");
        }
        let path_with_network = format!("{cln_dir}/{}/config", self.config.network);
        log::info!("configure coffee in the following cln config {path_with_network}");
        self.config.cln_config_path = Some(path_with_network);
        self.config.cln_root = Some(cln_dir.to_owned());
        self.load_cln_conf().await?;
        let mut conf = self.cln_config.clone().unwrap();
        conf.add_subconf(self.coffee_cln_config.clone())
            .map_err(|err| error!("{}", &err.cause))?;
        conf.flush()?;
        Ok(())
    }
}

#[async_trait]
impl PluginManager for CoffeeManager {
    async fn configure(&mut self) -> Result<(), CoffeeError> {
        log::debug!("plugin configured");
        Ok(())
    }

    async fn install(
        &mut self,
        plugin: &str,
        verbose: bool,
        try_dynamic: bool,
    ) -> Result<(), CoffeeError> {
        log::debug!("installing plugin: {plugin}");
        // keep track if the plugin is successfully installed
        for repo in self.repos.values() {
            if let Some(mut plugin) = repo.get_plugin_by_name(plugin) {
                log::trace!("{:?}", plugin);

                // old_root_path is the path where the plugin is cloned and currently stored
                // eg. ~/.coffee/repositories/<repo_name>/<plugin_name>
                let old_root_path = plugin.root_path.clone();
                // new_root_path is the path where the plugin will be installed specific to the network
                // eg. ~/.coffee/<network>/plugins/<plugin_name>
                let new_root_path = format!(
                    "{}/{}/plugins/{}",
                    self.config.root_path,
                    self.config.network,
                    plugin.name()
                );

                log::debug!(
                    "Start! copying directory from {} inside the new one {}",
                    old_root_path,
                    new_root_path
                );
                let script = format!("cp -r {old_root_path} {new_root_path}");
                sh!(self.config.root_path.clone(), script, verbose);
                log::debug!(
                    "Done! copying directory from {} inside the new one {}",
                    old_root_path,
                    new_root_path
                );
                let old_exec_path = plugin.exec_path.clone();

                match old_exec_path.strip_prefix(&old_root_path) {
                    Some(relative_path) => {
                        let new_exec_path = format!("{}{}", new_root_path, relative_path);
                        plugin.root_path = new_root_path;
                        plugin.exec_path = new_exec_path;

                        log::debug!("plugin: {:?}", plugin);
                        let path = plugin.configure(verbose).await?;
                        log::debug!("runnable plugin path {path}");
                        if !try_dynamic {
                            self.config.plugins.push(plugin);
                            log::debug!("path coffee conf: {}", self.coffee_cln_config.path);
                            self.coffee_cln_config
                                .add_conf("plugin", &path.to_owned())
                                .map_err(|err| error!("{}", err.cause))?;
                            log::debug!("coffee conf updated: {}", self.coffee_cln_config);
                            self.flush().await?;
                            self.update_conf().await?;
                        } else {
                            self.start_plugin(&path).await?;
                        }
                        return Ok(());
                    }
                    None => return Err(error!("exec path not found")),
                };
            }
        }
        Err(error!(
            "plugin `{plugin}` are not present inside the repositories"
        ))
    }

    async fn remove(&mut self, plugin: &str) -> Result<CoffeeRemove, CoffeeError> {
        log::debug!("removing plugin: {plugin}");
        let plugins = &mut self.config.plugins;
        if let Some(index) = plugins.iter().position(|x| x.name() == plugin) {
            let plugin = plugins[index].clone();
            let exec_path = plugin.exec_path.clone();
            let root_path = plugin.root_path.clone();
            let cloned_repositories_path = format!("{}/repositories", self.config.root_path,);
            // make sure that we are not deleting the cloned repositories
            if !root_path.contains(&cloned_repositories_path) {
                fs::remove_dir_all(root_path).await?;
            }
            log::debug!("runnable plugin path: {exec_path}");
            plugins.remove(index);
            log::debug!("coffee cln config: {}", self.coffee_cln_config);
            self.coffee_cln_config
                .rm_conf("plugin", Some(&exec_path.to_owned()))
                .map_err(|err| error!("{}", &err.cause))?;
            self.flush().await?;
            self.update_conf().await?;
            Ok(CoffeeRemove { plugin })
        } else {
            return Err(error!("plugin `{plugin}` is already not installed"));
        }
    }

    async fn list(&mut self) -> Result<CoffeeList, CoffeeError> {
        Ok(CoffeeList {
            plugins: self.config.plugins.clone(),
        })
    }

    async fn upgrade(&mut self, repo: &str) -> Result<CoffeeUpgrade, CoffeeError> {
        // TODO: upgrade should now be able to upgrade a single plugin
        // without affecting other plugins installed from the same repo
        let repository = self
            .repos
            .get_mut(repo)
            .ok_or_else(|| error!("Repository with name: `{}` not found", repo))?;

        let status = repository.upgrade(&self.config.plugins).await?;
        for plugins in status.plugins_effected.iter() {
            self.remove(plugins).await?;
            // FIXME: pass the verbose flag to the upgrade command
            self.install(plugins, false, false).await?;
        }
        self.flush().await?;
        Ok(status)
    }

    async fn setup(&mut self, cln_dir: &str) -> Result<(), CoffeeError> {
        self.setup_with_cln(cln_dir).await?;
        log::info!("cln configured");
        self.flush().await?;
        Ok(())
    }

    async fn add_remote(&mut self, name: &str, url: &str) -> Result<(), CoffeeError> {
        // FIXME: we should allow some error here like
        // for the add remote command the no found error for the `repository`
        // directory is fine.

        if self.repos.contains_key(name) {
            return Err(error!("repository with name: {name} already exists"));
        }
        let url = URL::new(&self.config.root_path, url, name);
        log::debug!("remote adding: {} {}", name, &url.url_string);
        let mut repo = Github::new(name, &url);
        repo.init().await?;
        self.repos.insert(repo.name(), Box::new(repo));
        log::debug!("remote added: {} {}", name, &url.url_string);
        self.flush().await?;
        Ok(())
    }

    async fn rm_remote(&mut self, name: &str) -> Result<(), CoffeeError> {
        log::debug!("remote removing: {}", name);
        match self.repos.get(name) {
            Some(repo) => {
                let remote_repo = repo.list().await?;
                let repo_path = repo.url().path_string;
                let plugins = self.config.plugins.clone();
                for plugin in &remote_repo {
                    if let Some(ind) = plugins
                        .iter()
                        .position(|elem| elem.name() == *plugin.name())
                    {
                        let plugin_name = &plugins[ind].name().clone();
                        match self.remove(plugin_name).await {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                }
                fs::remove_dir_all(repo_path).await?;
                self.repos.remove(name);
                log::debug!("remote removed: {}", name);
                self.flush().await?;
            }
            None => {
                return Err(error!("repository with name: {name} not found"));
            }
        };
        Ok(())
    }

    async fn list_remotes(&mut self) -> Result<CoffeeRemote, CoffeeError> {
        let mut remote_list = Vec::new();
        for repo in self.repos.values() {
            let repository = git2::Repository::open(repo.url().path_string.as_str())
                .map_err(|err| error!("{}", err.message()))?;
            let (commit, date) = get_repo_info!(repository);
            remote_list.push(CoffeeListRemote {
                local_name: repo.name(),
                url: repo.url().url_string,
                plugins: repo.list().await?,
                commit_id: Some(commit),
                date: Some(date),
            });
        }
        Ok(CoffeeRemote {
            remotes: Some(remote_list),
        })
    }

    async fn show(&mut self, plugin: &str) -> Result<CoffeeShow, CoffeeError> {
        for repo in self.repos.values() {
            if let Some(plugin) = repo.get_plugin_by_name(plugin) {
                // FIXME: there are more README file options?
                let readme_path = format!("{}/README.md", plugin.root_path);
                let contents = fs::read_to_string(readme_path).await?;
                return Ok(CoffeeShow { readme: contents });
            }
        }
        let err = error!(
            "{}",
            &format!("plugin `{plugin}` are not present inside the repositories"),
        );
        Err(err)
    }

    async fn search(&mut self, plugin: &str) -> Result<CoffeeSearch, CoffeeError> {
        for repo in self.repos.values() {
            if let Some(plugin) = repo.get_plugin_by_name(plugin) {
                return Ok(CoffeeSearch {
                    repository_url: repo.url().url_string,
                    plugin,
                });
            }
        }
        let err = CoffeeError::new(404, &format!("unable to locate plugin `{plugin}`"));
        Err(err)
    }

    async fn nurse(&mut self) -> Result<CoffeeNurse, CoffeeError> {
        let status = self.recovery_strategies.scan(self).await?;
        let mut nurse_actions: Vec<NurseStatus> = vec![];
        for defect in status.defects.iter() {
            log::debug!("defect: {:?}", defect);
            match defect {
                Defect::RepositoryLocallyAbsent(repos) => {
                    let mut actions = self.patch_repository_locally_absent(repos.to_vec()).await?;
                    nurse_actions.append(&mut actions);
                }
            }
        }

        // If there was no actions taken by nurse, we return a sane status.
        if nurse_actions.is_empty() {
            nurse_actions.push(NurseStatus::Sane);
        }
        Ok(CoffeeNurse {
            status: nurse_actions,
        })
    }

    async fn patch_repository_locally_absent(
        &mut self,
        repos: Vec<String>,
    ) -> Result<Vec<NurseStatus>, CoffeeError> {
        // initialize the nurse actions
        let mut nurse_actions: Vec<NurseStatus> = vec![];
        // for every repository that is absent locally
        // we try to recover it.
        // There are 2 cases:
        // 1. the repository can be recovered from the remote
        // 2. the repository can't be recovered from the remote. In this case
        //   we remove the repository from the coffee configuration.
        for repo_name in repos.iter() {
            // Get the repository from the name
            let repo = self
                .repos
                .get_mut(repo_name)
                .ok_or_else(|| error!("repository with name: {repo_name} not found"))?;

            match repo.recover().await {
                Ok(_) => {
                    log::info!("repository {} recovered", repo_name.clone());
                    nurse_actions.push(NurseStatus::RepositoryLocallyRestored(vec![
                        repo_name.clone()
                    ]));
                }
                Err(err) => {
                    log::debug!("error while recovering repository {repo_name}: {err}");
                    log::info!("removing repository {}", repo_name.clone());
                    self.repos.remove(repo_name);
                    log::debug!("remote removed: {}", repo_name);
                    self.flush().await?;
                    nurse_actions.push(NurseStatus::RepositoryLocallyRemoved(vec![
                        repo_name.clone()
                    ]));
                }
            }
        }
        Ok(nurse_actions)
    }
}

// FIXME: we need to move on but this is not safe and with the coffee
// implementation is not true!
unsafe impl Send for CoffeeManager {}
unsafe impl Sync for CoffeeManager {}
unsafe impl Send for CoffeeStorageInfo {}
unsafe impl Sync for CoffeeStorageInfo {}
