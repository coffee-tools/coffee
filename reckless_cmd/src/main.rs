mod reckless;

use crate::reckless::cmd::RecklessArgs;
use clap::Parser;
use reckless::{
    cmd::{RecklessCommand, RemoteAction},
    RecklessManager,
};
use reckless_lib::{errors::RecklessError, plugin_manager::PluginManager};

#[tokio::main]
async fn main() -> Result<(), RecklessError> {
    env_logger::init();
    let args = RecklessArgs::parse();
    let mut reckless = RecklessManager::new(&args).await?;
    let result = match args.command {
        RecklessCommand::Install => reckless.install(&[""]).await,
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
