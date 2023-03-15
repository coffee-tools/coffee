//! Coffee mod implementation
use coffee_cmd::cmd::CoffeeArgs;
use crate::config::CoffeeConf;
use async_trait::async_trait;
use clightningrpc_common::client::Client;
use clightningrpc_common::json_utils;
use clightningrpc_conf::{CLNConf, SyncCLNConf};
use coffee_github::repository::Github;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin::Plugin;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::repository::Repository;
use coffee_lib::url::URL;
use coffee_storage::file::FileStorage;
use coffee_storage::model::repository::{Kind, Repository as RepositoryInfo};
use coffee_storage::storage::StorageManager;
use log::{debug, error, info, trace, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::fmt::Debug;
use std::vec::Vec;

//use super::cmd;
use super::config;

#[derive(Serialize, Deserialize)]
/// FIXME: move the list of plugin
/// and the list of repository inside this struct.
pub struct CoffeStorageInfo {
    pub config: config::CoffeeConf,
    pub repositories: Vec<RepositoryInfo>,
}

impl From<&CoffeeManager> for CoffeStorageInfo {
    fn from(value: &CoffeeManager) -> Self {
        let mut repos = vec![];
        // FIXME: use map instead of for each
        // FIXME: improve the down cast
        value.repos.iter().for_each(|repo| {
            let repo = if let Some(git) = repo.as_any().downcast_ref::<Github>() {
                RepositoryInfo::from(git)
            } else {
                panic!("this should never happens")
            };
            repos.push(repo);
        });
        CoffeStorageInfo {
            config: value.config.to_owned(),
            repositories: repos, // FIXME: found a way to downcast
        }
    }
}

pub struct CoffeeManager {
    config: config::CoffeeConf,
    repos: Vec<Box<dyn Repository + Send + Sync>>,
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
    pub async fn new(conf: &CoffeeArgs) -> Result<Self, CoffeeError> {
        let conf = CoffeeConf::new(conf).await?;
        let mut coffee = CoffeeManager {
            config: conf.clone(),
            coffe_cln_config: CLNConf::new(conf.config_path, true),
            repos: vec![],
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
        let store = if let Ok(store) = self.storage.load().await {
            store
        } else {
            info!("storage file do not exist");
            return Ok(());
        };
        // this is really needed? I think no, because coffee at this point
        // have a new conf loading
        self.config = store.config;
        store.repositories.iter().for_each(|repo| match repo.kind {
            Kind::Git => {
                let repo = Github::from(repo);
                self.repos.push(Box::new(repo));
            }
        });
        if let Err(err) = self.coffe_cln_config.parse() {
            error!("{}", err.cause);
        }
        self.load_cln_conf().await?;
        debug!("cln conf {:?}", self.coffe_cln_config);
        debug!("finish pligin manager inventory");
        // FIXME: what are the information missed that
        // needed to be indexed?
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
            trace!("cln answer with {:#?}", response);
            if let Some(err) = response.error {
                return Err(CoffeeError::new(1, &format!("cln error: {}", err.message)));
            }
            return Ok(response.result.unwrap());
        }
        Err(CoffeeError::new(
            1,
            "rpc connection to core lightning not available",
        ))
    }

    pub async fn start_pluing(&self, path: &str) -> Result<(), CoffeeError> {
        let mut payload = json_utils::init_payload();
        json_utils::add_str(&mut payload, "subcommand", "start");
        json_utils::add_str(&mut payload, "plugin", path);
        let response = self
            .cln::<serde_json::Value, serde_json::Value>("plugin", payload)
            .await?;
        debug!("plugin registered: {response}");
        Ok(())
    }

    pub fn storage_info(&self) -> CoffeStorageInfo {
        CoffeStorageInfo::from(self)
    }

    pub async fn update_conf(&self) -> Result<(), CoffeeError> {
        self.coffe_cln_config.flush()?;
        debug!("stored all the cln info in {}", self.coffe_cln_config);
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
        info!("looking for the cln config: {path}");
        file.parse()
            .map_err(|err| CoffeeError::new(err.core, &err.cause))?;
        debug!("{:#?}", file.fields);
        self.cln_config = Some(file);
        Ok(())
    }

    pub async fn setup_with_cln(&mut self, cln_dir: &str) -> Result<(), CoffeeError> {
        if !self.cln_config.is_none() {
            warn!("you are ovveriding the previous set up");
        }
        let path_with_network = format!("{cln_dir}/{}/config", self.config.network);
        info!("configure coffe in the following cln config {path_with_network}");
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
        debug!("plugin configured");
        Ok(())
    }

    async fn install(
        &mut self,
        plugin: &str,
        verbose: bool,
        try_dynamic: bool,
    ) -> Result<(), CoffeeError> {
        debug!("installing plugin: {plugin}");
        // keep track if the plugin that are installed with success
        for repo in &self.repos {
            if let Some(mut plugin) = repo.get_plugin_by_name(plugin) {
                trace!("{:#?}", plugin);
                let result = plugin.configure(verbose).await;
                match result {
                    Ok(path) => {
                        debug!("runnable plugin path {path}");
                        if !try_dynamic {
                            self.config.plugins.push(plugin);
                            self.coffe_cln_config
                                .add_conf("plugin", &path.to_owned())
                                .map_err(|err| CoffeeError::new(1, &err.cause))?;

                            self.storage.store(&self.storage_info()).await?;
                            self.update_conf().await?;
                        } else {
                            self.start_pluing(&path).await?;
                        }
                        return Ok(());
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        let err = CoffeeError::new(
            1,
            &format!("plugin `{plugin}` are not present inside the repositories"),
        );
        Err(err)
    }

    async fn list(&mut self, remotes: bool) -> Result<Value, CoffeeError> {
        let installed_plugins_vec: Vec<Plugin> = self.config.plugins.clone();
        let plugin_json;
        if remotes {
            let mut remote_list: Vec<Value> = Vec::new();
            for repo in &self.repos {
                let remote_repo_json = json!(
                {
                    "local_name": repo.name(),
                    "link": repo.url().url_string,
                    "plugins": repo.list().await.unwrap(),
                 });
                remote_list.push(remote_repo_json);
            }
            plugin_json = json!({
               "plugins": serde_json::to_value(&installed_plugins_vec).unwrap(),
               "remotes": remote_list,
            });
        } else {
            plugin_json = json!({
               "plugins": serde_json::to_value(&installed_plugins_vec).unwrap(),
            });
        }
        Ok(plugin_json)
    }

    async fn upgrade(&mut self, _: &[&str]) -> Result<(), CoffeeError> {
        // FIXME: Fix debug message with the list of plugins to be upgraded
        debug!("upgrading plugins");
        Ok(())
    }

    async fn setup(&mut self, cln_dir: &str) -> Result<(), CoffeeError> {
        self.setup_with_cln(cln_dir).await?;
        info!("cln configured");
        self.storage.store(&self.storage_info()).await
    }

    async fn add_remote(&mut self, name: &str, url: &str) -> Result<(), CoffeeError> {
        let url = URL::new(&self.config.root_path, url, name);
        debug!("remote adding: {} {}", name, &url.url_string);
        let mut repo = Github::new(name, &url);
        repo.init().await?;
        self.repos.push(Box::new(repo));
        debug!("remote added: {} {}", name, &url.url_string);
        self.storage.store(&self.storage_info()).await?;
        Ok(())
    }
}

// FIXME: we need to move on but this is not safe and with the coffee
// implementation is not true!
unsafe impl Send for CoffeeManager {}
unsafe impl Sync for CoffeeManager {}
unsafe impl Send for CoffeStorageInfo {}
unsafe impl Sync for CoffeStorageInfo {}
