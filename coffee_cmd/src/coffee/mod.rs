//! Coffee mod implementation
use coffee_lib::url::URL;
use coffee_storage::file::FileStorage;
use coffee_storage::storage::StorageManager;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::vec::Vec;

use async_trait::async_trait;
use coffee_github::repository::Github;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::repository::Repository;

use self::cmd::CoffeeArgs;
use self::config::CoffeeConf;

pub mod cmd;
mod config;

#[derive(Serialize, Deserialize)]
/// FIXME: move the list of plugin
/// and the list of repository inside this struct.
pub struct CoffeStorageInfo {}

pub struct CoffeeManager {
    config: config::CoffeeConf,
    /// List of repositories
    repos: Vec<Box<dyn Repository + Send + Sync>>,
    /// List of plugins installed
    plugins: Vec<String>,
    /// storage instance to make persistent all the
    /// plugin manager information on disk
    storage: Box<dyn StorageManager<CoffeStorageInfo, Err = CoffeeError> + Send + Sync>,
}

impl CoffeeManager {
    pub async fn new(conf: &CoffeeArgs) -> Result<Self, CoffeeError> {
        let conf = CoffeeConf::new(conf).await?;
        let mut coffee = CoffeeManager {
            config: conf.clone(),
            repos: vec![],
            plugins: vec![],
            storage: Box::new(FileStorage::new(&conf.root_path)),
        };
        coffee.inventory().await?;
        Ok(coffee)
    }

    /// when coffee is configure run an inventory to collect all the necessary information
    /// about the coffee ecosystem.
    async fn inventory(&mut self) -> Result<(), CoffeeError> {
        let store: CoffeStorageInfo = self.storage.load().await?;
        // FIXME: bind the information from the storage
        // FIXME: what are the information missed that
        // needed to be indexed?
        Ok(())
    }

    pub fn storage_info(&self) -> CoffeStorageInfo {
        todo!()
    }
}

#[async_trait]
impl PluginManager for CoffeeManager {
    async fn configure(&mut self) -> Result<(), CoffeeError> {
        debug!("plugin configured");
        Ok(())
    }

    async fn install(&mut self, plugins: &HashSet<String>) -> Result<(), CoffeeError> {
        debug!("installing plugins {:?}", plugins);

        // keep track if the plugin that are installed with success
        let mut installed = HashSet::new();
        for repo in &self.repos {
            for plugin_name in plugins {
                if installed.contains(plugin_name) {
                    continue;
                }
                if let Some(mut plugin) = repo.get_plugin_by_name(&plugin_name) {
                    let result = plugin.configure().await;
                    match result {
                        Ok(path) => {
                            debug!("runnable plugin path {path}");
                            self.config.plugins_path.push(path);
                            installed.insert(plugin_name);
                            continue;
                        }
                        Err(err) => return Err(err),
                    }
                }

                // if we install all the plugin we return Ok
                if plugins.len() == installed.len() {
                    self.storage.store(&self.storage_info()).await?;
                    return Ok(());
                }
            }
        }

        // FIXME: improve the solution there, we can use the filter method
        let mut missed_plugins = vec![];
        for plugin_name in plugins {
            if !installed.contains(plugin_name) {
                missed_plugins.push(plugin_name);
            }
        }
        let err = CoffeeError::new(
            1,
            &format!(
                "plugin {:?} are not present inside the repositories",
                missed_plugins
            ),
        );
        Err(err)
    }

    async fn list(&mut self) -> Result<(), CoffeeError> {
        Ok(())
    }

    async fn upgrade(&mut self, plugins: &[&str]) -> Result<(), CoffeeError> {
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
        Ok(())
    }
}

// FIXME: we need to move on but this is not safe and with the coffee
// implementation is not true!
unsafe impl Send for CoffeeManager {}
unsafe impl Sync for CoffeeManager {}
unsafe impl Send for CoffeStorageInfo {}
unsafe impl Sync for CoffeStorageInfo {}
