mod reckless;

use crate::reckless::cmd::RecklessArgs;
use clap::Parser;
use reckless::{cmd::RecklessCommand, RecklessManager};
use reckless_lib::{errors::RecklessError, plugin_manager::PluginManager};

#[tokio::main]
async fn main() -> Result<(), RecklessError> {
    let args = RecklessArgs::parse();
    let mut reckless = RecklessManager::new(&args).await?;
    let result = match args.command {
        RecklessCommand::Install => reckless.install(&[""]).await,
        RecklessCommand::Remove => todo!(),
        RecklessCommand::List => reckless.list().await,
        RecklessCommand::Upgrade => reckless.upgrade(&[""]).await,
    };

    if let Err(err) = result {
        panic!("{err}");
    }

    Ok(())
}
