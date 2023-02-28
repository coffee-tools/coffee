mod coffee;

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
        CoffeeCommand::Install { plugin } => coffee.install(&plugin).await,
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
        CoffeeCommand::Setup { cln_conf } => {
            // FIXME: read the core lightning confi and
            // and the coffee script
            coffee.setup(&cln_conf).await
        }
    };

    if let Err(err) = result {
        panic!("{err}");
    }

    Ok(())
}
