//! storage defines the interface of the storage
//! manager that need to be implemented in other
//! to work with the the plugin manager
//! architecture.
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait StorageManager {
    type Err;

    /// async call to persist the information
    /// on disk.
    async fn store<T>(&self, key: &str, to_store: &T) -> Result<(), Self::Err>
    where
        T: Serialize + Send + Sync;

    /// async call to load the data that was made persistent
    /// from the previous `store` call.
    async fn load<T>(&self, key: &str) -> Result<T, Self::Err>
    where
        T: DeserializeOwned + Send + Sync;
}
