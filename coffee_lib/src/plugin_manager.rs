//! Plugin manager module definition.
use async_trait::async_trait;
use serde_json::Value;

use crate::errors::CoffeeError;

/// Plugin manager traits that define the API a generic
/// plugin manager.
#[async_trait]
pub trait PluginManager {
    /// configure the plugin manger.
    async fn configure(&mut self) -> Result<(), CoffeeError>;

    /// install a plugin by name, return an error if some error happens.
    async fn install(
        &mut self,
        plugins: &str,
        verbose: bool,
        try_dynamic: bool,
    ) -> Result<(), CoffeeError>;

    /// return the list of plugins installed by the plugin manager.
    async fn list(&mut self, remotes: bool) -> Result<Value, CoffeeError>;

    /// upgrade a sequence of plugin managed by the plugin manager.
    async fn upgrade(&mut self, plugins: &[&str]) -> Result<(), CoffeeError>;

    /// add the remote repository to the plugin manager.
    async fn add_remote(&mut self, name: &str, url: &str) -> Result<(), CoffeeError>;

    /// set up the core lightning configuration target for the
    /// plugin manager.
    async fn setup(&mut self, cln_conf_path: &str) -> Result<(), CoffeeError>;
}
