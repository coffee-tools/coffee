use async_trait::async_trait;
use coffee_lib::utils::check_dir_or_make_if_missing;
use nosql_db::NoSQL;
use nosql_sled::sled;
use nosql_sled::SledDB;

use coffee_lib::error;
use coffee_lib::errors::CoffeeError;

use crate::storage::StorageManager;

/// No SQL database
pub struct NoSQlStorage {
    inner: SledDB,
}

impl NoSQlStorage {
    pub async fn new(path: &str) -> Result<Self, CoffeeError> {
        let path = format!("{path}/storage");
        check_dir_or_make_if_missing(path.clone()).await?;
        let config = sled::Config::new().path(path).cache_capacity(1_000_000);
        let db = SledDB::try_from(config).map_err(|err| error!("{err}"))?;
        Ok(Self { inner: db })
    }
}

#[async_trait]
impl StorageManager for NoSQlStorage {
    type Err = CoffeeError;

    async fn load<T>(&self, key: &str) -> Result<T, Self::Err>
    where
        T: serde::de::DeserializeOwned + Send + Sync,
    {
        if !self.inner.contains(key) {
            return Err(error!(
                "value with key `{key}` not found inside the database"
            ));
        }
        let value = self.inner.get(key).map_err(|err| error!("{err}"))?;
        let value: T = serde_json::from_str(&value).map_err(|err| error!("{err}"))?;
        Ok(value)
    }

    async fn store<T>(&self, key: &str, to_store: &T) -> Result<(), Self::Err>
    where
        T: serde::Serialize + Send + Sync,
    {
        let value = serde_json::to_string(to_store).map_err(|err| error!("{err}"))?;
        self.inner.put(key, &value).map_err(|err| error!("{err}"))?;
        Ok(())
    }
}
