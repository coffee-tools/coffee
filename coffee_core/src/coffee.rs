//! Coffee mod implementation
use coffee_storage::nosql_db::NoSQlStorage;
use std::collections::HashMap;
use std::fmt::Debug;
use std::vec::Vec;
use tokio::fs;

use async_trait::async_trait;
use clightningrpc_common::client::Client;
use clightningrpc_common::json_utils;
use clightningrpc_conf::{CLNConf, SyncCLNConf};
use log;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use coffee_github::repository::Github;
use coffee_lib::error;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::repository::Repository;
use coffee_lib::types::response::*;
use coffee_lib::url::URL;
use coffee_storage::model::repository::{Kind, Repository as RepositoryInfo};
use coffee_storage::storage::StorageManager;

use super::config;
use crate::config::CoffeeConf;
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
    config: config::CoffeeConf,
    #[serde(skip_serializing, skip_deerializing)]
    repos: HashMap<String, Box<dyn Repository + Send + Sync>>,
    /// Core lightning configuration managed by coffee
    coffee_cln_config: CLNConf,
    /// Core lightning configuration that include the
    /// configuration managed by coffee
    cln_config: Option<CLNConf>,
    /// storage instance to make all the plugin manager
    /// information persistent on disk
    storage: Box<dyn StorageManager<CoffeeStorageInfo, Err = CoffeeError> + Send + Sync>,
    /// core lightning rpc connection
    rpc: Option<Client>,
}

impl CoffeeManager {
    pub async fn new(conf: &dyn CoffeeArgs) -> Result<Self, CoffeeError> {
        let conf = CoffeeConf::new(conf).await?;
        let mut coffee = CoffeeManager {
            config: conf.clone(),
            coffee_cln_config: CLNConf::new(conf.config_path, true),
            repos: HashMap::new(),
            storage: Box::new(NoSQlStorage::new(&conf.root_path).await?),
            cln_config: None,
            rpc: None,
        };
        coffee.inventory().await?;
        Ok(coffee)
    }

    /// when coffee is configured, run an inventory to collect all the necessary information
    /// about the coffee ecosystem.
    async fn inventory(&mut self) -> Result<(), CoffeeError> {
        let Ok(store) = self.storage.load(&self.config.network).await else {
            log::debug!("storage do not exist");
            return Ok(());
        };
        // this is really needed? I think no, because coffee at this point
        // have a new conf loading
        self.config = store.config;
        let repositories = self.storage.load::<>("repository")?;
        store
            .repositories
            .iter()
            .for_each(|repo| match repo.1.kind {
                Kind::Git => {
                    let repo = Github::from(repo.1);
                    self.repos.insert(repo.name(), Box::new(repo));
                }
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
                .map_err(|err| CoffeeError::new(1, &format!("{err}")))?;
            log::debug!("cln answer with {:?}", response);
            if let Some(err) = response.error {
                return Err(CoffeeError::new(1, &format!("cln error: {}", err.message)));
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
        self.storage
            .store(&self.config.network, &self.storage_info())
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
        log::debug!("{:#?}", file.fields);
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
            .map_err(|err| CoffeeError::new(1, &err.cause))?;
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
                log::trace!("{:#?}", plugin);
                let result = plugin.configure(verbose).await;
                log::debug!("result from plugin configure: {:?}", result);
                match result {
                    Ok(path) => {
                        log::debug!("runnable plugin path {path}");
                        if !try_dynamic {
                            self.config.plugins.push(plugin);
                            log::debug!("path coffee conf: {}", self.coffee_cln_config.path);
                            self.coffee_cln_config
                                .add_conf("plugin", &path.to_owned())
                                .map_err(|err| CoffeeError::new(1, &err.cause))?;
                            log::debug!("coffee conf updated: {}", self.coffee_cln_config);
                            self.flush().await?;
                            self.update_conf().await?;
                        } else {
                            self.start_plugin(&path).await?;
                        }
                        return Ok(());
                    }
                    Err(err) => return Err(err),
                }
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
            log::debug!("runnable plugin path: {exec_path}");
            plugins.remove(index);
            log::debug!("coffee cln config: {}", self.coffee_cln_config);
            self.coffee_cln_config
                .rm_conf("plugin", Some(&exec_path.to_owned()))
                .map_err(|err| CoffeeError::new(1, &err.cause))?;
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
        let repository = self
            .repos
            .get(repo)
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
            remote_list.push(CoffeeListRemote {
                local_name: repo.name(),
                url: repo.url().url_string,
                plugins: repo.list().await?,
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
        let err = CoffeeError::new(
            1,
            &format!("plugin `{plugin}` are not present inside the repositories"),
        );
        Err(err)
    }

    async fn nurse(&mut self) -> Result<(), CoffeeError> {
        unimplemented!("nurse command is not implemented")
    }
}

// FIXME: we need to move on but this is not safe and with the coffee
// implementation is not true!
unsafe impl Send for CoffeeManager {}
unsafe impl Sync for CoffeeManager {}
unsafe impl Send for CoffeeStorageInfo {}
unsafe impl Sync for CoffeeStorageInfo {}
