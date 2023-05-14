mod cmd;
mod coffee_term;

use clap::Parser;
use radicle_term as term;

use coffee_core::coffee::CoffeeManager;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::types::{NurseStatus, UpgradeStatus};

use crate::cmd::CoffeeArgs;
use crate::cmd::CoffeeCommand;
use crate::cmd::RemoteAction;

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
        CoffeeCommand::Remove { plugin } => {
            let mut spinner = term::spinner(format!("Uninstalling plugin {plugin}"));
            let result = coffee.remove(&plugin).await;
            if let Err(err) = &result {
                spinner.error(format!("Error while uninstalling the plugin: {err}"));
                return Ok(());
            }
            spinner.message("Plugin uninstalled!");
            spinner.finish();
            Ok(())
        }
        CoffeeCommand::List {} => {
            let remotes = coffee.list().await;
            coffee_term::show_list(remotes)
        }
        CoffeeCommand::Upgrade { repo, all } => {
            let result = match repo {
                Some(repo) => coffee.upgrade(&repo, all).await,
                None => coffee.upgrade("", all).await,
            };
            match result {
                Ok(val) => {
                    for res in val.total_status {
                        match res.status {
                            UpgradeStatus::UpToDate => {
                                term::info!("Remote repository {} is already up to date!", res.repo)
                            }
                            UpgradeStatus::Updated => term::success!(
                                "Remote repository {} was successfully upgraded!",
                                res.repo
                            ),
                        }
                    }
                }
                Err(err) => term::error(format!("{err}")),
            }
            Ok(())
        }
        CoffeeCommand::Remote { action } => match action {
            RemoteAction::Add { name, url } => {
                let mut spinner = term::spinner(format!("Fetch remote from {url}"));
                let result = coffee.add_remote(&name, &url).await;
                if let Err(err) = &result {
                    spinner.error(format!("Error while add remote: {err}"));
                    return result;
                }
                spinner.message("Remote added!");
                spinner.finish();
                Ok(())
            }
            RemoteAction::Rm { name } => {
                let mut spinner = term::spinner(format!("Removing remote {name}"));
                let result = coffee.rm_remote(&name).await;
                if let Err(err) = &result {
                    spinner.error(format!("Error while removing the repository: {err}"));
                    return result;
                }
                spinner.message("Remote removed!");
                spinner.finish();
                Ok(())
            }
            RemoteAction::List {} => {
                let remotes = coffee.list_remotes().await;
                coffee_term::show_remote_list(remotes)
            }
        },
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
        CoffeeCommand::Nurse {} => {
            let mut spinner =
                term::spinner("Trying restoring from a corrupted status!".to_string());
            match coffee.nurse().await {
                Ok(val) => {
                    match val.status {
                        NurseStatus::Corrupted => spinner.message("Storage refreshed"),
                        NurseStatus::Sane => spinner
                            .message("Storage files are not corrupt. No need to run coffee nurse"),
                    }
                    spinner.finish();
                }
                Err(err) => spinner.error(format!("Error while refreshing storage: {err}")),
            };
            Ok(())
        }
    };

    if let Err(err) = result {
        term::error(format!("{err}"));
    }

    Ok(())
}
