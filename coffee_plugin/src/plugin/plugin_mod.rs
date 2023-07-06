//! Coffee plugin implementation to use
//! Coffee as a core lightning plugin.
use std::fmt::Display;

use serde_json::{json, Value};
use tokio::runtime::Runtime;

use clightningrpc_common::json_utils;
use clightningrpc_plugin::commands::RPCCommand;
use clightningrpc_plugin::plugin::{debug, info};
use clightningrpc_plugin::{add_rpc, error};
use clightningrpc_plugin::{errors::PluginError, plugin::Plugin};

use coffee_core::coffee::CoffeeManager;
use coffee_lib::errors::CoffeeError;
use coffee_lib::macros::error as coffee_err;
use coffee_lib::plugin_manager::PluginManager;

use super::model::{InstallReq, RemoteCmd, RemoteReq};
use super::state::PluginArgs;
use crate::plugin::State;

pub fn build_plugin() -> Result<Plugin<State>, PluginError> {
    let mut plugin = Plugin::<State>::new(State::new(), /* dynamic */ true).on_init(on_init);
    add_rpc!(plugin, CoffeeInstall);
    add_rpc!(plugin, CoffeeList);
    add_rpc!(plugin, CoffeeRemote);
    Ok(plugin)
}

/// on init function is called by the plugin workflow when the
/// init method is sent from core lightning
///
/// This is an interceptor, at this point the plugin configuration and
/// options are already binding with the plugin.
fn on_init(plugin: &mut Plugin<State>) -> Value {
    let response = json_utils::init_payload();
    let cln_conf = plugin.configuration.clone().unwrap();
    let args = PluginArgs::from(cln_conf);
    info!("{:?}", args);
    plugin.state.set_args(args);

    debug!("{:?}", plugin.configuration);
    debug!("{:?}", plugin.state.args);

    // Get the runtime and start the block function
    let runtime = Runtime::new().unwrap();
    let result = runtime.block_on(async move {
        let coffee = CoffeeManager::new(&plugin.state.args()).await;
        if let Err(err) = &coffee {
            debug!("{err}");
            return Err(coffee_err!("{err}"));
        }
        let coffee = coffee.unwrap();
        plugin.state.set_coffee(coffee);
        plugin.state.setup().await
    });

    if let Err(err) = result {
        let err = format!("{err}");
        return json!({
            "disable": err,
        });
    }

    response
}

fn from<T: Display>(err: T) -> PluginError {
    error!("{err}")
}

#[derive(Clone)]
struct CoffeeInstall {
    name: String,
    usage: String,
    description: String,
}

impl CoffeeInstall {
    fn new() -> Self {
        CoffeeInstall {
            name: "coffee_install".to_string(),
            usage: String::new(),
            description: String::from("install a plugin from one of the repository choosed"),
        }
    }
}

impl RPCCommand<State> for CoffeeInstall {
    fn call<'c>(
        &self,
        plugin: &mut Plugin<State>,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError> {
        let coffee = plugin.state.coffee();
        let mut coffee = coffee.lock().unwrap();
        let rt = Runtime::new().unwrap();

        let request: InstallReq = serde_json::from_value(request)?;
        rt.block_on(coffee.install(&request.name, false, true))
            .map_err(from)?;
        Ok(json!({}))
    }
}

#[derive(Clone)]
struct CoffeeList {
    name: String,
    usage: String,
    description: String,
}

impl CoffeeList {
    fn new() -> Self {
        CoffeeList { name: "coffee_list".to_owned(), usage: String::new(), description: "show all the plugin installed and if {remotes} is specified show also the one available".to_owned() }
    }
}

impl RPCCommand<State> for CoffeeList {
    fn call<'c>(
        &self,
        plugin: &mut Plugin<State>,
        _: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError> {
        let runtime = Runtime::new().unwrap();
        let coffee = plugin.state.coffee();
        let mut coffee = coffee.lock().unwrap();
        let result = runtime.block_on(coffee.list()).map_err(from)?;
        Ok(serde_json::to_value(result)?)
    }
}

#[derive(Clone)]
struct CoffeeRemote {
    name: String,
    usage: String,
    description: String,
}

impl CoffeeRemote {
    fn new() -> Self {
        CoffeeRemote {
            name: "coffee_remote".to_owned(),
            usage: String::new(),
            description: "manage a remote".to_owned(),
        }
    }
}

impl RPCCommand<State> for CoffeeRemote {
    fn call<'c>(
        &self,
        plugin: &mut Plugin<State>,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PluginError> {
        let request: RemoteReq = serde_json::from_value(request)?;
        let runtime = Runtime::new().unwrap();
        let coffee = plugin.state.coffee();

        runtime
            .block_on(async {
                let mut coffee = coffee.lock().unwrap();
                let cmd = request.cmd().unwrap();
                match cmd {
                    RemoteCmd::Add => coffee.add_remote(&request.name, &request.url()).await,
                    RemoteCmd::Rm => coffee.rm_remote(&request.name).await,
                }
            })
            .map_err(from)?;
        Ok(json!({}))
    }
}
