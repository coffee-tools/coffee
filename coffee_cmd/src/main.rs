mod cmd;
mod coffee_term;

use clap::Parser;
use radicle_term as term;

use coffee_core::coffee::CoffeeManager;
use coffee_lib::error;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::types::response::{CoffeeRemote, UpgradeStatus};

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
        CoffeeCommand::Upgrade { repo, verbose } => {
            match coffee.upgrade(&repo, verbose).await {
                Ok(res) => match res.status {
                    UpgradeStatus::UpToDate => {
                        term::info!("Remote repository `{}` is already up to date!", res.repo)
                    }
                    UpgradeStatus::Updated => {
                        term::success!(
                            "Remote repository `{}` was successfully upgraded!",
                            res.repo
                        )
                    }
                },
                Err(err) => return Err(err),
            }
            Ok(())
        }
        CoffeeCommand::Remote {
            action,
            plugins,
            name,
        } => {
            if plugins {
                let result = coffee.get_plugins_in_remote(&name.unwrap()).await;
                coffee_term::show_list(result)
            } else {
                match action {
                    Some(RemoteAction::Add { name, url }) => {
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
                    Some(RemoteAction::Rm { name }) => {
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
                    Some(RemoteAction::List {}) => {
                        let remotes = coffee.list_remotes().await;
                        coffee_term::show_remote_list(remotes)
                    }
                    None => {
                        // This is the case when the user does not provides the
                        // plugins flag, so we just show the remote repository
                        // information

                        // The name will be always Some because of the
                        // arg_required_else_help = true in the clap
                        // attribute
                        let name =
                            name.ok_or_else(|| error!("No remote repository name provided"))?;
                        let remotes = coffee.list_remotes().await?;
                        let remotes = remotes
                            .remotes
                            .ok_or_else(|| error!("Couldn't get the remote repositories"))?;
                        let remote = remotes
                            .iter()
                            .find(|remote| remote.local_name == name)
                            .ok_or_else(|| error!("Couldn't find the remote repository"))?;
                        // A workaround to show the remote repository information
                        // in the same way as the list command
                        let remote = Ok(CoffeeRemote {
                            remotes: Some(vec![remote.clone()]),
                        });
                        coffee_term::show_remote_list(remote)
                    }
                }
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
                let val = val.readme.as_str();
                term::markdown(val);
                Ok(())
            }
            Err(err) => Err(err),
        },
        CoffeeCommand::Search { plugin } => match coffee.search(&plugin).await {
            Ok(val) => {
                let repository_url = val.repository_url.as_str();
                term::success!("found plugin {plugin} in remote repository {repository_url}");
                Ok(())
            }
            Err(err) => Err(err),
        },
        CoffeeCommand::Nurse { verify } => {
            if verify {
                let result = coffee.nurse_verify().await?;
                term::info!("{}", result);
                if !result.is_sane() {
                    term::info!("Coffee local directory is damaged, please run `coffee nurse` to try to fix it");
                }
                Ok(())
            } else {
                let nurse_result = coffee.nurse().await;
                coffee_term::show_nurse_result(nurse_result)
            }
        }
    };

    if let Err(err) = result {
        term::error(format!("{err}"));
    }

    Ok(())
}
