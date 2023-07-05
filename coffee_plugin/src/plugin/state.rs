//! Coffee State struct implementation
use std::sync::Arc;
use std::sync::Mutex;

use clightningrpc_plugin::commands::types::CLNConf;
use coffee_core::{coffee::CoffeeManager, CoffeeArgs};
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;

#[derive(Clone)]
pub struct State {
    pub coffee: Option<Arc<Mutex<CoffeeManager>>>,
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
        self.coffee = Some(Arc::new(Mutex::new(coffee)));
    }

    pub fn coffee(&self) -> Arc<Mutex<CoffeeManager>> {
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

    pub async fn setup(&self) -> Result<(), CoffeeError> {
        self.coffee()
            .lock()
            .unwrap()
            .setup(&self.args.clone().unwrap().conf)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
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
        let mut root = value.lightning_dir.as_str();
        if value.lightning_dir.ends_with("/testnet") {
            root = value.lightning_dir.strip_suffix("/testnet").unwrap();
        } else if value.lightning_dir.ends_with("/bitcoin") {
            root = value.lightning_dir.strip_suffix("/bitcoin").unwrap();
        } else if value.lightning_dir.ends_with("/regtest") {
            root = value.lightning_dir.strip_suffix("/regtest").unwrap();
        }
        PluginArgs {
            conf: root.to_owned(),
            network: value.network,
            data_dir: Some(root.to_owned()),
        }
    }
}
