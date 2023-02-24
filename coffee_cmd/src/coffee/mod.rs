//! Coffee mod implementation
use self::cmd::CoffeeArgs;
use self::config::CoffeeConf;
use async_trait::async_trait;
use clightningrpc_conf::{CLNConf, SyncCLNConf};
use coffee_github::repository::Github;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::repository::Repository;
use coffee_lib::url::URL;
use coffee_storage::file::FileStorage;
use coffee_storage::model::repository::{Kind, Repository as RepositoryInfo};
use coffee_storage::storage::StorageManager;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::vec::Vec;

pub mod cmd;
mod config;

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
    /// List of repositories
    repos: Vec<Box<dyn Repository + Send + Sync>>,
    /// Core lightning configuration
    cln_config: CLNConf,
    /// storage instance to make persistent all the
    /// plugin manager information on disk
    storage: Box<dyn StorageManager<CoffeStorageInfo, Err = CoffeeError> + Send + Sync>,
}

impl CoffeeManager {
    pub async fn new(conf: &CoffeeArgs) -> Result<Self, CoffeeError> {
        let conf = CoffeeConf::new(conf).await?;
        let mut coffee = CoffeeManager {
            config: conf.clone(),
            cln_config: CLNConf::new(conf.cln_config_path, true),
            repos: vec![],
            storage: Box::new(FileStorage::new(&conf.root_path)),
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
        if let Err(err) = self.cln_config.parse() {
            error!("{}", err.cause);
        }
        debug!("cln conf {:?}", self.cln_config);
        debug!("finish pligin manager inventory");
        // FIXME: what are the information missed that
        // needed to be indexed?
        Ok(())
    }

    pub fn storage_info(&self) -> CoffeStorageInfo {
        CoffeStorageInfo::from(self)
    }

    pub async fn update_cln_conf(&self) -> Result<(), CoffeeError> {
        self.cln_config.flush()?;
        debug!("stored all the cln info in {}", self.cln_config);
        Ok(())
    }
}

#[async_trait]
impl PluginManager for CoffeeManager {
    async fn configure(&mut self) -> Result<(), CoffeeError> {
        debug!("plugin configured");
        Ok(())
    }

    async fn install(&mut self, plugin: &str) -> Result<(), CoffeeError> {
        debug!("installing plugin: {plugin}");
        // keep track if the plugin that are installed with success
        for repo in &self.repos {
            if let Some(mut plugin) = repo.get_plugin_by_name(plugin) {
                let result = plugin.configure().await;
                match result {
                    Ok(path) => {
                        debug!("runnable plugin path {path}");
                        self.config.plugins_path.push(path.to_string());
                        self.cln_config.add_conf("plugin", &path.to_owned());

                        self.storage.store(&self.storage_info()).await?;
                        self.update_cln_conf().await?;
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

    async fn list(&mut self) -> Result<(), CoffeeError> {
        Ok(())
    }

    async fn upgrade(&mut self, _: &[&str]) -> Result<(), CoffeeError> {
        // FIXME: Fix debug message with the list of plugins to be upgraded
        debug!("upgrading plugins");
        Ok(())
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
