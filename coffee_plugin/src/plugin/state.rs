//! Coffee State struct implementation
use std::sync::Arc;

use clightningrpc_plugin::commands::types::CLNConf;
use coffee_core::{coffee::CoffeeManager, CoffeeArgs};

#[derive(Clone)]
pub struct State {
    pub coffee: Option<Arc<CoffeeManager>>,
    pub args: Option<PluginArgs>,
}

impl State {
    pub fn new() -> Self {
        State {
            coffee: None,
            args: None,
        }
    }

    pub fn set_coffee(&mut self, coffee: CoffeeManager) {
        self.coffee = Some(Arc::new(coffee));
    }

    #[allow(dead_code)]
    pub fn coffee(&self) -> Arc<CoffeeManager> {
        self.coffee.clone().unwrap()
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

impl From<CLNConf> for PluginArgs {
    fn from(value: CLNConf) -> Self {
        PluginArgs {
            conf: value.lightning_dir,
            network: value.network,
            data_dir: None,
        }
    }
}
