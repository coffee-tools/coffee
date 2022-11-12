//! Repository module implementation that contains all the code to build a repository
//! for a plugin manager.
use crate::errors::RecklessError;
use crate::plugin::Plugin;

use async_trait::async_trait;

#[async_trait]
pub trait Repository {
    /// init the plugin manager repository in local
    /// machine.
    ///
    /// This should work like a `git fetch`.
    async fn init(&self) -> Result<(), RecklessError>;

    /// return the list of plugin that are register contained inside the repository.
    async fn list(&self) -> Result<Vec<Plugin>, RecklessError>;
}
