//! Coffee testing implementation!
pub mod btc;
pub mod cln;

pub mod prelude {
    pub use crate::macros::*;
    pub use port_selector as port;
    pub use tempfile;
}
use std::sync::Arc;

use port_selector as port;
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
            use $crate::DEFAULT_TIMEOUT;

            $crate::wait_for!($callback, DEFAULT_TIMEOUT);
        };
    }

    #[macro_export]
    macro_rules! httpd {
        ($dir:expr, $port:expr, $($opt_args:tt)*) => {
            async {
                use std::process::Stdio;

                use tokio::process::Command;

                let opt_args = format!($($opt_args)*);
                let args = opt_args.trim();
                let args_tok: Vec<&str> = args.split(" ").collect();

                let cargo_target = std::env::var("CARGO_TARGET_DIR").unwrap();
                let httpd_path = format!("{cargo_target}/debug/coffee_httpd");
                let mut command = Command::new(httpd_path);
                command
                    .args(&args_tok)
                    .arg("--host=127.0.0.1")
                    .arg(format!("--port={}", $port))
                    .arg(format!("--data-dir={}", $dir.path().to_str().unwrap()))
                    .stdout(Stdio::null())
                    .spawn()
            }.await
        };
        ($dir:expr, $port:expr) => {
            $crate::lightningd!($dir, $port, "")
        };
    }

    pub use httpd;
    pub use wait_for;
}

pub struct CoffeeTestingArgs {
    pub conf: Option<String>,
    pub network: String,
    pub data_dir: String,
}

unsafe impl Send for CoffeeTestingArgs {}
unsafe impl Sync for CoffeeTestingArgs {}

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
    /// init coffee in a tmp directory.
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
            httpd_pid: None,
            httpd_port: None,
        })
    }

    pub fn coffee(&mut self) -> &mut CoffeeManager {
        &mut self.inner
    }

    pub fn root_path(&self) -> Arc<TempDir> {
        self.root_path.clone()
    }

    /// run the httpd deamon as process and return the URL
    /// this should allow to make integration testing to the httpd.
    pub async fn httpd(&mut self) -> anyhow::Result<String> {
        let port = port::random_free_port().unwrap();
        let child = httpd!(self.root_path, port, "--network=regtest")?;
        self.httpd_port = Some(port.into());
        self.httpd_pid = Some(child);
        Ok(format!("127.0.0.1:{port}"))
    }
}
