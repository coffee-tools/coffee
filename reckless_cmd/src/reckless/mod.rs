//! Reckless mod implementation
use coffe_storage::file::FileStorage;
use coffe_storage::storage::StorageManager;
use log::debug;
use reckless_lib::url::URL;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::vec::Vec;

use async_trait::async_trait;
use reckless_github::repository::Github;
use reckless_lib::errors::RecklessError;
use reckless_lib::plugin_manager::PluginManager;
use reckless_lib::repository::Repository;

use self::cmd::RecklessArgs;
use self::config::RecklessConf;

pub mod cmd;
mod config;

#[derive(Serialize, Deserialize)]
/// FIXME: move the list of plugin
/// and the list of repository inside this struct.
pub struct CoffeStorageInfo {}

pub struct RecklessManager {
    config: config::RecklessConf,
    /// List of repositories
    repos: Vec<Box<dyn Repository + Send + Sync>>,
    /// List of plugins installed
    plugins: Vec<String>,
    /// storage instance to make persistent all the
    /// plugin manager information on disk
    storage: Box<dyn StorageManager<CoffeStorageInfo, Err = RecklessError> + Send + Sync>,
}

impl RecklessManager {
    pub async fn new(conf: &RecklessArgs) -> Result<Self, RecklessError> {
        let mut reckless = RecklessManager {
            config: RecklessConf::new(conf).await?,
            repos: vec![],
            plugins: vec![],
            // FIXME: store the path from the conf inside the
            // storage, this is needed to get the position
            // where to store the disk info
            storage: Box::new(FileStorage {}),
        };
        reckless.inventory().await?;
        Ok(reckless)
    }

    /// when reckless is configure run an inventory to collect all the necessary information
    /// about the reckless ecosystem.
    async fn inventory(&mut self) -> Result<(), RecklessError> {
        let mut stored = CoffeStorageInfo {};
        self.storage.load(&mut stored).await?;
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
impl PluginManager for RecklessManager {
    async fn configure(&mut self) -> Result<(), RecklessError> {
        debug!("plugin configured");
        Ok(())
    }

    async fn install(&mut self, plugins: &HashSet<String>) -> Result<(), RecklessError> {
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
        let err = RecklessError::new(
            1,
            &format!(
                "plugin {:?} are not present inside the repositories",
                missed_plugins
            ),
        );
        Err(err)
    }

    async fn list(&mut self) -> Result<(), RecklessError> {
        Ok(())
    }

    async fn upgrade(&mut self, plugins: &[&str]) -> Result<(), RecklessError> {
        // FIXME: Fix debug message with the list of plugins to be upgraded
        debug!("upgrading plugins");
        Ok(())
    }

    async fn add_remote(&mut self, name: &str, url: &str) -> Result<(), RecklessError> {
        let url = URL::new(&self.config.root_path, url, name);
        debug!("remote adding: {} {}", name, &url.url_string);
        let mut repo = Github::new(name, &url);
        repo.init().await?;
        self.repos.push(Box::new(repo));
        debug!("remote added: {} {}", name, &url.url_string);
        Ok(())
    }
}

// FIXME: we need to move on but this is not safe and with the reckless
// implementation is not true!
unsafe impl Send for RecklessManager {}
unsafe impl Sync for RecklessManager {}
unsafe impl Send for CoffeStorageInfo {}
unsafe impl Sync for CoffeStorageInfo {}
