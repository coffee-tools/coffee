//! Coffee testing implementation!
pub mod btc;
pub mod cln;

pub mod prelude {
    pub use cln_btc_test;
    pub use cln_test;

    pub use crate::macros::*;
    pub use tempfile;
}
use std::sync::Arc;

use tempfile::TempDir;

use coffee_core::coffee::CoffeeManager;

static DEFAULT_TIMEOUT: u64 = 100;

pub mod macros {
    #[macro_export]
    macro_rules! wait_for {
        ($callback:expr, $timeout:expr) => {
            use log;
            use tokio::time::{sleep, Duration};

            for wait in 0..$timeout {
                if let Err(err) = $callback.await {
                    log::debug!("callback return {:?}", err);
                    sleep(Duration::from_millis(wait)).await;
                    continue;
                }
                log::info!("callback completed in {wait} milliseconds");
                break;
            }
        };
        ($callback:expr) => {
            use crate::DEFAULT_TIMEOUT;

            $crate::wait_for!($callback, DEFAULT_TIMEOUT);
        };
    }

    pub use wait_for;
}

pub struct CoffeeTestingArgs {
    pub conf: Option<String>,
    pub network: String,
    pub data_dir: String,
}

impl coffee_core::CoffeeArgs for CoffeeTestingArgs {
    fn command(&self) -> coffee_core::CoffeeOperation {
        unimplemented!()
    }

    fn conf(&self) -> Option<String> {
        self.conf.clone()
    }

    fn data_dir(&self) -> Option<String> {
        Some(self.data_dir.clone())
    }

    fn network(&self) -> Option<String> {
        Some(self.network.clone())
    }
}

/// Coffee testing manager
/// that contains all the information that
/// we need to perform integration testing for coffee.
pub struct CoffeeTesting {
    inner: CoffeeManager,
    root_path: Arc<TempDir>,
}

impl CoffeeTesting {
    // init coffee in a tmp directory.
    pub async fn tmp() -> anyhow::Result<Self> {
        let dir = tempfile::tempdir()?;
        let args = CoffeeTestingArgs {
            data_dir: dir.path().to_str().unwrap().to_owned(),
            network: "regtest".to_owned(),
            conf: None,
        };
        let coffee = CoffeeManager::new(&args)
            .await
            .map_err(|err| anyhow::anyhow!("{err}"))?;
        Ok(Self {
            inner: coffee,
            root_path: Arc::new(dir),
        })
    }

    // init coffee in a tmp directory with arguments.
    pub async fn tmp_with_args(
        args: &CoffeeTestingArgs,
        tempdir: Arc<TempDir>,
    ) -> anyhow::Result<Self> {
        log::info!("Temporary directory: {:?}", tempdir);

        let coffee = CoffeeManager::new(args)
            .await
            .map_err(|err| anyhow::anyhow!("{err}"))?;

        Ok(Self {
            inner: coffee,
            root_path: tempdir,
        })
    }

    pub fn coffee(&mut self) -> &mut CoffeeManager {
        &mut self.inner
    }

    pub fn root_path(&self) -> Arc<TempDir> {
        self.root_path.clone()
    }
}
