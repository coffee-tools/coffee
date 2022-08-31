//! Plugin manager module definition.
use async_trait::async_trait;

use crate::errors::RecklessError;

/// Plugin manager traits that define the API a generic
/// plugin manager.
#[async_trait]
pub trait PluginManager {
    /// configure the plugin manger.
    async fn configure(&mut self) -> Result<(), RecklessError>;

    /// install a sequence of plugin or return an error if somethings happens.
    // FIXME: what happens if only one plugin fails?
    async fn install(&mut self, plugins: &[&str]) -> Result<(), RecklessError>;

    /// return the list of pluing manager by the plugin manager.
    async fn list(&mut self) -> Result<(), RecklessError>;

    /// upgrade a sequence of plugin managed by the plugin manager.
    async fn upgrade(&mut self, plugins: &[&str]) -> Result<(), RecklessError>;
}
