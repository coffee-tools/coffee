mod coffee;

use std::collections::HashSet;

use crate::coffee::cmd::CoffeeArgs;
use clap::Parser;
use coffee::cmd::CoffeeCommand;
use coffee::cmd::RemoteAction;
use coffee::CoffeeManager;

use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;

#[tokio::main]
async fn main() -> Result<(), CoffeeError> {
    env_logger::init();
    let args = CoffeeArgs::parse();
    let mut coffee = CoffeeManager::new(&args).await?;
    let result = match args.command {
        CoffeeCommand::Install { plugin } => {
            let mut unique_plugin: HashSet<String> = HashSet::new();
            plugin.iter().for_each(|plugin| {
                unique_plugin.insert(plugin.to_owned());
            });
            coffee.install(&unique_plugin).await
        }
        CoffeeCommand::Remove => todo!(),
        CoffeeCommand::List => coffee.list().await,
        CoffeeCommand::Upgrade => coffee.upgrade(&[""]).await,
        CoffeeCommand::Remote { action } => {
            if let RemoteAction::Add { name, url } = action {
                coffee.add_remote(name.as_str(), url.as_str()).await
            } else {
                Err(CoffeeError::new(1, "unsupported command"))
            }
        }
    };

    if let Err(err) = result {
        panic!("{err}");
    }

    Ok(())
}