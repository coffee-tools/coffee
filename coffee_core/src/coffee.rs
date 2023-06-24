//! Coffee mod implementation
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::vec::Vec;
use tokio::fs;

use async_trait::async_trait;
use clightningrpc_common::client::Client;
use clightningrpc_common::json_utils;
use clightningrpc_conf::{CLNConf, SyncCLNConf};
use log;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

use coffee_github::repository::Github;
use coffee_lib::error;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::repository::Repository;
use coffee_lib::types::{
    CoffeeList, CoffeeListRemote, CoffeeNurse, CoffeeRemote, CoffeeRemove, CoffeeUpgrade,
    NurseStatus,
};
use coffee_lib::url::URL;
use coffee_storage::file::FileStorage;
use coffee_storage::model::repository::{Kind, Repository as RepositoryInfo};
use coffee_storage::storage::StorageManager;

use super::config;
use crate::config::CoffeeConf;
use crate::CoffeeArgs;

pub type PluginName = String;

#[derive(Serialize, Deserialize)]
/// FIXME: move the list of plugin
/// and the list of repository inside this struct.
pub struct CoffeStorageInfo {
    pub config: config::CoffeeConf,
    pub repositories: HashMap<PluginName, RepositoryInfo>,
}

impl From<&CoffeeManager> for CoffeStorageInfo {
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

        CoffeStorageInfo {
            config: value.config.to_owned(),
            repositories: repos, // FIXME: found a way to downcast
        }
    }
}

pub struct CoffeeManager {
    config: config::CoffeeConf,
    repos: HashMap<String, Box<dyn Repository + Send + Sync>>,
    /// Core lightning configuration managed by coffee
    coffe_cln_config: CLNConf,
    /// Core lightning configuration that include the
    /// configuration managed by coffee
    cln_config: Option<CLNConf>,
    /// storage instance to make persistent all the
    /// plugin manager information on disk
    storage: Box<dyn StorageManager<CoffeStorageInfo, Err = CoffeeError> + Send + Sync>,
    /// core lightning rpc connection
    rpc: Option<Client>,
}

impl CoffeeManager {
    pub async fn new(conf: &dyn CoffeeArgs) -> Result<Self, CoffeeError> {
        let conf = CoffeeConf::new(conf).await?;
        let mut coffee = CoffeeManager {
            config: conf.clone(),
            coffe_cln_config: CLNConf::new(conf.config_path, true),
            repos: HashMap::new(),
            storage: Box::new(FileStorage::new(&conf.root_path)),
            cln_config: None,
            rpc: None,
        };
        coffee.inventory().await?;
        Ok(coffee)
    }

    /// when coffee is configure run an inventory to collect all the necessary information
    /// about the coffee ecosystem.
    async fn inventory(&mut self) -> Result<(), CoffeeError> {
        let Ok(store) = self.storage.load().await else {
            log::debug!("storage file do not exist");
            return Ok(());
        };
        // this is really needed? I think no, because coffee at this point
        // have a new conf loading
        self.config = store.config;
        store
            .repositories
            .iter()
            .for_each(|repo| match repo.1.kind {
                Kind::Git => {
                    let repo = Github::from(repo.1);
                    self.repos.insert(repo.name(), Box::new(repo));
                }
            });
        if let Err(err) = self.coffe_cln_config.parse() {
            log::error!("{}", err.cause);
        }
        self.load_cln_conf().await?;
        log::debug!("cln conf {:?}", self.coffe_cln_config);
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
            log::trace!("cln answer with {:#?}", response);
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

    pub fn storage_info(&self) -> CoffeStorageInfo {
        CoffeStorageInfo::from(self)
    }

    pub async fn update_conf(&self) -> Result<(), CoffeeError> {
        self.coffe_cln_config.flush()?;
        log::debug!("stored all the cln info in {}", self.coffe_cln_config);
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
            log::warn!("you are ovveriding the previous set up");
        }
        let path_with_network = format!("{cln_dir}/{}/config", self.config.network);
        log::info!("configure coffe in the following cln config {path_with_network}");
        self.config.cln_config_path = Some(path_with_network);
        self.config.cln_root = Some(cln_dir.to_owned());
        self.load_cln_conf().await?;
        let mut conf = self.cln_config.clone().unwrap();
        conf.add_subconf(self.coffe_cln_config.clone())
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
        self.remote_sync().await?;
        log::debug!("installing plugin: {plugin}");
        // keep track if the plugin that are installed with success
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
                            self.coffe_cln_config
                                .add_conf("plugin", &path.to_owned())
                                .map_err(|err| CoffeeError::new(1, &err.cause))?;

                            self.storage.store(&self.storage_info()).await?;
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
            self.coffe_cln_config
                .rm_conf("plugin", Some(&exec_path.to_owned()))
                .map_err(|err| CoffeeError::new(1, &err.cause))?;
            self.storage.store(&self.storage_info()).await?;
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
        self.remote_sync().await?;

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
        self.storage.store(&self.storage_info()).await?;
        Ok(status)
    }

    async fn setup(&mut self, cln_dir: &str) -> Result<(), CoffeeError> {
        self.setup_with_cln(cln_dir).await?;
        log::info!("cln configured");
        self.storage.store(&self.storage_info()).await
    }

    async fn remote_sync(&mut self) -> Result<(), CoffeeError> {
        // check if there are any unrelated files or folders in the `repositories` folder
        let repos_path = PathBuf::from(format!("{}/repositories/", self.config.root_path));
        let mut dir_entries = fs::read_dir(repos_path)
            .await
            .map_err(|_| CoffeeError::new(1, "`repositories` folder wasn't found"))?;
        while let Some(entry) = dir_entries.next_entry().await? {
            let entry_name = entry.file_name().to_string_lossy().to_string();
            if !self.repos.contains_key(&entry_name) {
                log::warn!("An unknown file or folder was detected in coffee local storage. Coffee is corrupted");
                return Err(error!(
                    "coffee storage is out of sync. Please run coffee nurse to resolve the issue"
                ));
            }
        }

        // check if a the whole remote repository clone was removed
        for (repo_name, repo) in self.repos.iter_mut() {
            let repo_path = repo.url().path_string;
            let repo_url = repo.url().url_string;
            if !Path::new(&repo_path).exists() {
                log::warn!("remote with name {repo_name} and URL {repo_url} is not longer present! Coffee is corrupted");
                return Err(error!(
                    "coffee storage is out of sync. Please run coffee nurse to resolve the issue"
                ));
            }
        }

        Ok(())
    }

    async fn add_remote(&mut self, name: &str, url: &str) -> Result<(), CoffeeError> {
        // FIXME: we should allow some error here like
        // for the add remote command the no found error for the `repository`
        // directory is fine.
        // self.remote_sync().await?;

        if self.repos.contains_key(name) {
            return Err(error!("repository with name: {name} already exists"));
        }
        let url = URL::new(&self.config.root_path, url, name);
        log::debug!("remote adding: {} {}", name, &url.url_string);
        let mut repo = Github::new(name, &url);
        repo.init().await?;
        self.repos.insert(repo.name(), Box::new(repo));
        log::debug!("remote added: {} {}", name, &url.url_string);
        self.storage.store(&self.storage_info()).await?;
        Ok(())
    }

    async fn rm_remote(&mut self, name: &str) -> Result<(), CoffeeError> {
        self.remote_sync().await?;
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
                self.storage.store(&self.storage_info()).await?;
            }
            None => {
                return Err(error!("repository with name: {name} not found"));
            }
        };
        Ok(())
    }

    async fn list_remotes(&mut self) -> Result<CoffeeRemote, CoffeeError> {
        self.remote_sync().await?;
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

    async fn show(&mut self, plugin: &str) -> Result<Value, CoffeeError> {
        self.remote_sync().await?;
        for repo in self.repos.values() {
            if let Some(plugin) = repo.get_plugin_by_name(plugin) {
                // FIXME: there are more README file options?
                let readme_path = format!("{}/README.md", plugin.root_path);
                let contents = fs::read_to_string(readme_path).await?;
                return Ok(json!({ "show": contents }));
            }
        }
        let err = CoffeeError::new(
            1,
            &format!("plugin `{plugin}` are not present inside the repositories"),
        );
        Err(err)
    }

    async fn nurse(&mut self) -> Result<CoffeeNurse, CoffeeError> {
        let mut status = NurseStatus::Sane;

        // delete any unrelated files or folders in `repositories` folder
        let repos_path = PathBuf::from(format!("{}/repositories/", self.config.root_path));
        let mut dir_entries = fs::read_dir(repos_path).await?;
        while let Some(entry) = dir_entries.next_entry().await? {
            let entry_name = entry.file_name().to_string_lossy().to_string();
            if !self.repos.contains_key(&entry_name) {
                // if there is an unrelated file or directory in repositories,
                // we consider the folder corrupted
                status = NurseStatus::Corrupted;
                if entry.file_type().await?.is_dir() {
                    fs::remove_dir_all(entry.path()).await?;
                    log::debug!("folder removed: {}", entry_name);
                } else if entry.file_type().await?.is_file() {
                    fs::remove_file(entry.path()).await?;
                    log::debug!("file removed: {}", entry_name);
                }
            }
        }

        // check if the existing local repositories clones are corrupt.
        // remove them from configuration if they are and remove the installed plugins
        let mut keys_to_remove: Vec<String> = Vec::new();
        for (repo_name, repo) in &self.repos {
            let repo_path = repo.url().path_string;
            if !Path::new(&repo_path).exists() {
                status = NurseStatus::Corrupted;
                keys_to_remove.push(repo_name.to_string());
            }
        }
        for repo_name in &keys_to_remove {
            let remote_repo = self.repos[repo_name].list().await?;
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
        }
        for key in keys_to_remove {
            self.repos.remove(&key);
            log::debug!("repository removed: {key}");
        }

        self.storage.store(&self.storage_info()).await?;
        Ok(CoffeeNurse { status })
    }
}

// FIXME: we need to move on but this is not safe and with the coffee
// implementation is not true!
unsafe impl Send for CoffeeManager {}
unsafe impl Sync for CoffeeManager {}
unsafe impl Send for CoffeStorageInfo {}
unsafe impl Sync for CoffeStorageInfo {}
