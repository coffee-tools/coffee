mod cmd;

use crate::cmd::CoffeeArgs;
use crate::cmd::CoffeeCommand;
use crate::cmd::RemoteAction;
use clap::Parser;
use radicle_term as term;

use coffee_core::coffee::CoffeeManager;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;

#[tokio::main]
async fn main() -> Result<(), CoffeeError> {
    env_logger::init();
    let args = CoffeeArgs::parse();
    let mut coffee = CoffeeManager::new(&args).await?;
    let result = match args.command {
        CoffeeCommand::Install {
            plugin,
            verbose,
            dynamic,
        } => {
            let spinner = if !verbose {
                Some(term::spinner("Compiling and installing"))
            } else {
                None
            };
            let result = coffee.install(&plugin, verbose, dynamic).await;
            if let Some(spinner) = spinner {
                if result.is_ok() {
                    spinner.finish();
                } else {
                    spinner.failed();
                }
            } else if result.is_ok() {
                term::success!("Plugin {plugin} Compiled and Installed")
            }
            result
        }
        CoffeeCommand::Remove => todo!(),
        CoffeeCommand::List { remotes } => match coffee.list(remotes).await {
            Ok(val) => {
                println!("{}", serde_json::to_string_pretty(&val).unwrap());
                Ok(())
            }
            Err(err) => Err(err),
        },
        CoffeeCommand::Upgrade => coffee.upgrade(&[""]).await,
        CoffeeCommand::Remote { action } => {
            if let RemoteAction::Add { name, url } = action {
                coffee.add_remote(name.as_str(), url.as_str()).await
            } else if let RemoteAction::Rm { name } = action {
                coffee.rm_remote(name.as_str()).await
            } else if let RemoteAction::List {} = action {
                match coffee.list_remotes().await {
                    Ok(val) => {
                        println!("{}", serde_json::to_string_pretty(&val).unwrap());
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            } else {
                Err(CoffeeError::new(1, "unsupported command"))
            }
        }
        CoffeeCommand::Setup { cln_conf } => {
            // FIXME: read the core lightning config
            // and the coffee script
            coffee.setup(&cln_conf).await
        }
        CoffeeCommand::Show { plugin } => match coffee.show(&plugin).await {
            Ok(val) => {
                // FIXME: modify the radicle_term markdown
                let val = val["show"].as_str().unwrap();
                term::markdown(val);
                Ok(())
            }
            Err(err) => Err(err),
        },
    };

    if let Err(err) = result {
        term::error(format!("{err}"));
    }

    Ok(())
}
