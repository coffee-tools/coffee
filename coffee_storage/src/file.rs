//! file is a module to store a simple
//! JSON file on the disk with a full
//! dump of the plugin manager status.
//!
//! This will work for the initial version
//! of it, but maybe in the future it is needed
//! a more smart version of storage manager
use crate::storage::StorageManager;
use async_trait::async_trait;
use coffee_lib::errors::CoffeeError;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub struct FileStorage {
    /// path of the storage file
    pub path: String,
    name_file: String,
}

impl FileStorage {
    pub fn new(path: &str) -> Self {
        FileStorage {
            path: path.to_owned(),
            name_file: "storage.json".to_owned(),
        }
    }

    pub fn get_path(&self) -> String {
        format!("{}/{}", self.path, self.name_file)
    }
}

#[async_trait]
impl<T> StorageManager<T> for FileStorage {
    type Err = CoffeeError;

    async fn load<'c>(&self) -> Result<T, Self::Err>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let mut content = String::new();
        File::open(self.get_path())
            .await?
            .read_to_string(&mut content)
            .await?;
        let val = serde_json::from_str::<T>(&content).unwrap();
        Ok(val)
    }

    async fn store(&self, to_store: &T) -> Result<(), Self::Err>
    where
        T: Serialize + Send + Sync,
    {
        let content = serde_json::to_string(to_store).unwrap();
        File::create(self.get_path())
            .await?
            .write_all(content.as_bytes())
            .await?;
        Ok(())
    }
}
