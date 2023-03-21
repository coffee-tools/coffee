//! Coffee plugin implementation to use
//! Coffee as a core lightning plugin.
use clightningrpc_common::json_utils;
use clightningrpc_plugin::types::LogLevel;
use clightningrpc_plugin::{errors::PluginError, plugin::Plugin};
use coffee_core::coffee::CoffeeManager;
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::plugin::State;

use super::state::PluginArgs;

pub async fn build_plugin() -> Result<Plugin<State>, PluginError> {
    let plugin = Plugin::<State>::new(State::new(), /* dynamic */ true).on_init(&on_init);
    Ok(plugin)
}

/// on init function called by the plugin workflow when the
/// init method is sent from core lightning
///
/// This is an interceptor, at this point the plugin configuration and
/// options are already binding with the plugin.
fn on_init(plugin: &mut Plugin<State>) -> Value {
    let response = json_utils::init_payload();
    let cln_conf = plugin.configuration.clone().unwrap();
    let args = PluginArgs::from(cln_conf);
    plugin.state.set_args(args);

    // Get the runtime and start the block function
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        let coffee = CoffeeManager::new(&plugin.state.args()).await;
        if let Err(err) = &coffee {
            plugin.log(LogLevel::Debug, &format!("{err}"));
        }
        plugin.state.set_coffee(coffee.unwrap());
    });

    response
}
