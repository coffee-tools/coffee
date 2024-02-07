//! Coffee plugin implementation to use
//! Coffee as a core lightning plugin.
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::runtime::Runtime;

use clightningrpc_common::json_utils;
use clightningrpc_plugin::commands::RPCCommand;
use clightningrpc_plugin::error;
use clightningrpc_plugin::plugin::{debug, info};
use clightningrpc_plugin::{errors::PluginError, plugin::Plugin};
use clightningrpc_plugin_macros::{plugin, rpc_method};

use coffee_core::coffee::CoffeeManager;
use coffee_lib::errors::CoffeeError;
use coffee_lib::macros::error as coffee_err;
use coffee_lib::plugin_manager::PluginManager;

use super::model::{InstallReq, RemoteCmd, RemoteReq};
use super::state::PluginArgs;
use crate::plugin::State;

pub fn build_plugin() -> Result<Plugin<State>, PluginError> {
    let mut plugin = plugin! {
        state: State::new(),
        dynamic: true,
        notification: [],
        methods: [
            coffee_install,
            coffee_list,
            coffee_remote,
            coffee_generate_tip,
        ],
        hooks: [],
    };
    plugin.on_init(on_init);
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

#[rpc_method(
    rpc_name = "coffee_install",
    description = "install a plugin from one of the repository choosed"
)]
fn coffee_install(plugin: &mut Plugin<State>, request: Value) -> Result<Value, PluginError> {
    let coffee = plugin.state.coffee();
    let mut coffee = coffee.lock().unwrap();
    let rt = Runtime::new().unwrap();

    let request: InstallReq = serde_json::from_value(request)?;
    rt.block_on(coffee.install(&request.name, false, true, Some(request.branch)))
        .map_err(from)?;
    Ok(json!({}))
}

#[rpc_method(
    rpc_name = "coffee_list",
    description = "show all the plugin installed and if {remotes} is specified show also the one available"
)]
fn coffee_list(plugin: &mut Plugin<State>, _: Value) -> Result<Value, PluginError> {
    let runtime = Runtime::new().unwrap();
    let coffee = plugin.state.coffee();
    let mut coffee = coffee.lock().unwrap();
    let result = runtime.block_on(coffee.list()).map_err(from)?;
    Ok(serde_json::to_value(result)?)
}

#[rpc_method(rpc_name = "coffee_remote", description = "manage a remote")]
fn coffee_remote(plugin: &mut Plugin<State>, request: Value) -> Result<Value, PluginError> {
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

#[rpc_method(
    rpc_name = "coffee_generate_tip",
    description = "Generate the BOLT 12 to add inside a plugin configuration to receive donation"
)]
fn coffee_generate_tip(plugin: &mut Plugin<State>, request: Value) -> Result<Value, PluginError> {
    let runtime = Runtime::new().unwrap();
    let coffee = plugin.state.coffee();

    #[derive(Serialize, Deserialize, Debug)]
    struct Offer {
        pub bolt12: String,
    }

    let offer = runtime
        .block_on(async {
            let mut coffee = coffee.lock().unwrap();
            coffee.cln::<Value, Offer>("offer", json!({
                "amount": "any",
                "description": "Generating BOLT 12 for coffee tips regarding the plugin ...",
            })).await
        })
        .map_err(from)?;
    Ok(serde_json::to_value(offer)?)
}
