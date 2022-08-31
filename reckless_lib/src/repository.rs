//! Repository module implementation that contains all the code to build a repository
//! for a plugin manager.
use crate::errors::RecklessError;
use crate::plugin::Plugin;

use async_trait::async_trait;

#[async_trait]
pub trait Repository<'tcx> {
    /// init the plugin manager repository in local
    /// machine.
    ///
    /// This should work like a `git fetch`.
    async fn init() -> &'tcx Self;

    /// return the list of plugin that are register contained inside the repository.
    async fn list() -> Result<Vec<&'tcx Plugin>, RecklessError>;
}
