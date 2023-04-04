//! Coffee plugin implementation to use
//! Coffee as a core lightning plugin.
use cln_plugin::Builder;
use cln_plugin::ConfiguredPlugin;
use cln_plugin::Plugin;
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite};

use coffee_core::coffee::CoffeeManager;
use coffee_core::{CoffeeError, PluginManager};

use super::model::{RemoteCmd, RemoteReq};
use super::state::PluginArgs;

use crate::plugin::model::InstallReq;
use crate::plugin::State;

pub async fn build_plugin() -> Result<(), anyhow::Error> {
    let plugin = match Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .dynamic()
        .rpcmethod("coffee_remote", "add a new remote plugin", remote)
        .rpcmethod("coffee_install", "install a new plugin", install)
        .configure()
        .await?
    {
        Some(p) => p,
        None => return Ok(()),
    };
    let result = on_init(&plugin).await;

    if let Err(_) = result {
        plugin.disable("an error occurs").await?;
        return Ok(());
    };

    plugin.start(result.unwrap()).await?.join().await?;
    Ok(())
}

/// on init function called by the plugin workflow when the
/// init method is sent from core lightning
///
/// This is an interceptor, at this point the plugin configuration and
/// options are already binding with the plugin.
async fn on_init<I, O>(plugin: &ConfiguredPlugin<State, I, O>) -> Result<State, CoffeeError>
where
    I: AsyncRead + Send + Unpin + 'static,
    O: Send + AsyncWrite + Unpin + 'static,
{
    let cln_conf = plugin.configuration();
    let args = PluginArgs::from(cln_conf.clone());

    // do something async
    let coffee = CoffeeManager::new(&args).await;
    if let Err(err) = coffee {
        return Err(err);
    }
    let mut state = State::new(coffee.unwrap());
    state.set_args(args);
    Ok(state)
}

async fn remote(
    plugin: Plugin<State>,
    request: serde_json::Value,
) -> Result<serde_json::Value, anyhow::Error> {
    let mut coffee = plugin.state().coffee().await;
    let request: RemoteReq = serde_json::from_value(request)?;

    let result = match request.cmd().unwrap() {
        RemoteCmd::Add => coffee.add_remote(&request.name, &request.url()).await,
        RemoteCmd::Rm => coffee.rm_remote(&request.name).await,
    };
    if let Err(err) = result {
        anyhow::bail!("{err}");
    }
    Ok(json!({}))
}

async fn install(
    plugin: Plugin<State>,
    request: serde_json::Value,
) -> Result<serde_json::Value, anyhow::Error> {
    let mut coffee = plugin.state().coffee().await;
    let request: InstallReq = serde_json::from_value(request)?;
    let result = coffee.install(&request.name, false, true).await;
    if let Err(err) = result {
        anyhow::bail!("{err}");
    }
    Ok(json!({}))
}
