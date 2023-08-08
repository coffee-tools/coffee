//! Repository module implementation that contains all the code to build a repository
//! for a plugin manager.
use std::any::Any;

use crate::errors::CoffeeError;
use crate::plugin::Plugin;
use crate::url::URL;

use crate::types::response::CoffeeUpgrade;

use async_trait::async_trait;

#[async_trait]
pub trait Repository: Any {
    /// init the plugin manager repository in local
    /// machine.
    ///
    /// This should work like a `git fetch`.
    async fn init(&mut self) -> Result<(), CoffeeError>;

    /// search inside the repository a plugin by name.
    fn get_plugin_by_name(&self, name: &str) -> Option<Plugin>;

    /// return the list of plugin that are register contained inside the repository.
    async fn list(&self) -> Result<Vec<Plugin>, CoffeeError>;

    /// upgrade the repository
    async fn upgrade(&mut self, plugins: &Vec<Plugin>) -> Result<CoffeeUpgrade, CoffeeError>;

    /// return the name of the repository.
    fn name(&self) -> String;

    /// return the url of the repository.
    fn url(&self) -> URL;

    fn as_any(&self) -> &dyn Any;
}
