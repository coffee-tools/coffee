mod cmd;

use crate::cmd::CoffeeArgs;
use crate::cmd::CoffeeCommand;
use crate::cmd::RemoteAction;
use clap::Parser;
use coffee_core::coffee::CoffeeManager;
use coffee_lib::errors::CoffeeError;
use coffee_lib::plugin_manager::PluginManager;

use termimad::crossterm::style::Color::*;
use termimad::{Alignment, MadSkin};

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
        } => coffee.install(&plugin, verbose, dynamic).await,
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
            } else {
                Err(CoffeeError::new(1, "unsupported command"))
            }
        }
        CoffeeCommand::Setup { cln_conf } => {
            // FIXME: read the core lightning confi and
            // and the coffee script
            coffee.setup(&cln_conf).await
        }
        CoffeeCommand::Show { plugin } => match coffee.show(&plugin).await {
            Ok(val) => {
                let mut skin = MadSkin::default();
                skin.table.align = Alignment::Center;
                skin.set_headers_fg(AnsiValue(178));
                skin.bold.set_fg(Yellow);
                skin.italic.set_fg(Magenta);
                skin.scrollbar.thumb.set_fg(AnsiValue(178));
                skin.code_block.align = Alignment::Center;

                skin.print_text(val["show"].as_str().unwrap());
                Ok(())
            }
            Err(err) => Err(err),
        },
    };

    if let Err(err) = result {
        panic!("{err}");
    }

    Ok(())
}
