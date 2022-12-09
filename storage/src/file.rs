//! file is a module to store a simple
//! JSON file on the disk with a full
//! dump of the plugin manager status.
//!
//! This will work for the initial version
//! of it, but maybe in the future it is needed
//! a more smart version of storage manager
use crate::storage::StorageManager;
use async_trait::async_trait;
use reckless_lib::{errors::RecklessError, plugin_manager::PluginManager};
use serde::{Deserialize, Serialize};

pub struct FileStorage {}

#[async_trait]
impl<T> StorageManager<T> for FileStorage {
    type Err = RecklessError;

    async fn load(&self, to_load: &mut T) -> Result<(), Self::Err>
    where
        T: Deserialize<'static> + Send + Sync,
    {
        Ok(())
    }

    async fn store(&self, to_store: &T) -> Result<(), Self::Err>
    where
        T: Serialize + Send + Sync,
    {
        Ok(())
    }
}
