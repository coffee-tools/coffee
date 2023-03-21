#![feature(async_closure)]
mod plugin;
use clightningrpc_plugin::errors::PluginError;
use plugin::build_plugin;

#[tokio::main]
async fn main() -> Result<(), PluginError> {
    let plugin = build_plugin().await?;
    plugin.start();
    Ok(())
}
