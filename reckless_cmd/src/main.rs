mod reckless;

use std::collections::HashSet;

use crate::reckless::cmd::RecklessArgs;
use clap::Parser;
use reckless::cmd::RecklessCommand;
use reckless::cmd::RemoteAction;
use reckless::RecklessManager;

use reckless_lib::errors::RecklessError;
use reckless_lib::plugin_manager::PluginManager;

#[tokio::main]
async fn main() -> Result<(), RecklessError> {
    env_logger::init();
    let args = RecklessArgs::parse();
    let mut reckless = RecklessManager::new(&args).await?;
    let result = match args.command {
        RecklessCommand::Install { plugin } => {
            let mut unique_plugin: HashSet<String> = HashSet::new();
            plugin
                .iter()
                .map(|plugin| unique_plugin.insert(plugin.to_owned()));
            reckless.install(&unique_plugin).await
        }
        RecklessCommand::Remove => todo!(),
        RecklessCommand::List => reckless.list().await,
        RecklessCommand::Upgrade => reckless.upgrade(&[""]).await,
        RecklessCommand::Remote { action } => {
            if let RemoteAction::Add { name, url } = action {
                reckless.add_remote(name.as_str(), url.as_str()).await
            } else {
                Err(RecklessError::new(1, "unsupported command"))
            }
        }
    };

    if let Err(err) = result {
        panic!("{err}");
    }

    Ok(())
}
