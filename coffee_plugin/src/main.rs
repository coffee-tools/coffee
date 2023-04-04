mod plugin;
use clightningrpc_plugin::errors::PluginError;
use plugin::build_plugin;

fn main() -> Result<(), PluginError> {
    let plugin = build_plugin();
    plugin?.start();
    Ok(())
}
