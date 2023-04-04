//! Coffe State stuct implemenation
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use cln_plugin::messages::Configuration;
use coffee_core::{coffee::CoffeeManager, CoffeeArgs};

#[derive(Clone)]
pub struct State {
    pub coffee: Arc<Mutex<CoffeeManager>>,
    pub args: Option<PluginArgs>,
}

impl State {
    pub fn new(coffee: CoffeeManager) -> Self {
        State {
            coffee: Arc::new(Mutex::new(coffee)),
            args: None,
        }
    }

    pub async fn coffee(&self) -> MutexGuard<'_, CoffeeManager> {
        self.coffee.lock().await
    }

    pub fn set_args(&mut self, args: PluginArgs) {
        self.args = Some(args);
    }

    /// return the args when set, otherwise panic
    ///
    /// This must be call after the called use `set_args` to
    /// init the internal state.
    pub fn args(&self) -> PluginArgs {
        self.args.clone().unwrap()
    }
}

#[derive(Clone)]
pub struct PluginArgs {
    pub conf: String,
    pub network: String,
    pub data_dir: Option<String>,
    // FIXME: support datadir
}

impl CoffeeArgs for PluginArgs {
    fn command(&self) -> coffee_core::CoffeeOperation {
        unimplemented!()
    }

    fn conf(&self) -> Option<String> {
        Some(self.conf.clone())
    }

    fn data_dir(&self) -> Option<String> {
        self.data_dir.clone()
    }

    fn network(&self) -> Option<String> {
        Some(self.network.clone())
    }
}

impl From<Configuration> for PluginArgs {
    fn from(value: Configuration) -> Self {
        PluginArgs {
            conf: value.lightning_dir,
            network: value.network,
            data_dir: None,
        }
    }
}
