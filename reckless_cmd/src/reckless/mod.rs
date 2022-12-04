//! Reckless mod implementation
use log::debug;
use reckless_lib::url::URL;
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

pub struct RecklessManager {
    config: config::RecklessConf,
    /// List of repositories
    repos: Vec<Box<Github>>,
    /// List of plugins installed
    plugins: Vec<String>,
}

impl RecklessManager {
    pub async fn new(conf: &RecklessArgs) -> Result<Self, RecklessError> {
        let mut reckless = RecklessManager {
            config: RecklessConf::new(conf).await?,
            repos: vec![],
            plugins: vec![],
        };
        reckless.inventory().await?;
        Ok(reckless)
    }

    /// when reckless is configure run an inventory to collect all the necessary information
    /// about the reckless ecosystem.
    async fn inventory(&mut self) -> Result<(), RecklessError> {
        Ok(())
    }
}

#[async_trait]
impl PluginManager for RecklessManager {
    async fn configure(&mut self) -> Result<(), RecklessError> {
        debug!("plugin configured");
        Ok(())
    }

    async fn install(&mut self, plugins: &Vec<String>) -> Result<(), RecklessError> {
        debug!("installing plugins {:?}", plugins);
        for plugin_to_be_installed in plugins {
            for repo in &self.repos {
                for plugin_list in repo.list().await {
                    for mut plugin in plugin_list {
                        if plugin_to_be_installed == &plugin.name() {
                            // FIXME: This path needs to be added to config
                            let path = plugin.configure().await.unwrap();
                            debug!("{path}");
                        }
                    }
                }
            }
        }
        Ok(())
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
        let url = URL::new(url, Some(name));
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
