//! Reckless mod implementation
use std::vec::Vec;

use async_trait::async_trait;
use reckless_lib::errors::RecklessError;
use reckless_lib::plugin_manager::PluginManager;

use self::cmd::RecklessArgs;
use self::config::RecklessConf;

pub mod cmd;
mod config;

pub struct RecklessManager {
    config: config::RecklessConf,
    repos: Vec<String>,
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
        Ok(())
    }

    async fn install(&mut self, plugins: &[&str]) -> Result<(), RecklessError> {
        Ok(())
    }

    async fn list(&mut self) -> Result<(), RecklessError> {
        Ok(())
    }

    async fn upgrade(&mut self, plugins: &[&str]) -> Result<(), RecklessError> {
        Ok(())
    }

    async fn add_remote(&mut self, name: &str, url: &str) -> Result<(), RecklessError> {
        // 1. create the repository
        // 2. init the repository
        // 3. if all is ok store the repository in the plugin manager
        Ok(())
    }
}
