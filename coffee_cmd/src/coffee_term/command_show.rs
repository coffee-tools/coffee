//! Implementing the code function to show
//! the command result on the terminal!

use radicle_term as term;
use term::table::TableOptions;
use term::Element;

use coffee_lib::error;
use coffee_lib::errors::CoffeeError;
use coffee_lib::types::response::{CoffeeList, CoffeeNurse, CoffeeRemote, CoffeeTip, NurseStatus};

pub fn show_list(coffee_list: Result<CoffeeList, CoffeeError>) -> Result<(), CoffeeError> {
    let remotes = coffee_list?;

    term::println(
        term::format::bold("●"),
        term::format::tertiary("Plugins installed"),
    );
    let mut table = radicle_term::Table::new(TableOptions::bordered());
    table.push([
        term::format::dim(String::from("●")),
        term::format::bold(String::from("Language")),
        term::format::bold(String::from("Name")),
        term::format::bold(String::from("Enabled?")),
        term::format::bold(String::from("Exec path")),
    ]);
    table.divider();

    for plugin in &remotes.plugins {
        // Get whether the plugin is enabled
        // If enabled is None, it means the plugin is enabled by default for backward compatibility.
        let enabled = plugin.enabled.unwrap_or(true);
        table.push([
            term::format::positive("●").into(),
            term::format::highlight(plugin.lang.to_string()),
            term::format::bold(plugin.name()),
            if enabled {
                term::format::positive("yes").into()
            } else {
                term::format::negative("no").into()
            },
            term::format::highlight(plugin.exec_path.to_owned()),
        ])
    }
    table.print();
    Ok(())
}

pub fn show_remote_list(remote_list: Result<CoffeeRemote, CoffeeError>) -> Result<(), CoffeeError> {
    let repositories = remote_list?.remotes;

    let Some(repositories) = repositories else {
        return Err(error!("the repository is empty"));
    };

    term::println(
        term::format::bold("●"),
        term::format::tertiary("List of repositories"),
    );
    let mut table = radicle_term::Table::new(TableOptions::bordered());
    table.push([
        term::format::dim(String::from("●")),
        term::format::bold(String::from("Repository Alias")),
        term::format::bold(String::from("URL")),
        term::format::bold(String::from("N. Plugins")),
        term::format::bold(String::from("Git HEAD")),
        term::format::bold(String::from("Last Update")),
    ]);
    table.divider();

    for repository in &repositories {
        let mut commit_id = repository.commit_id.clone().unwrap_or_default();
        commit_id = commit_id.chars().take(7).collect::<String>();
        let date = repository.date.clone().unwrap_or_default();
        table.push([
            term::format::positive("●").into(),
            term::format::highlight(repository.local_name.to_owned()),
            term::format::bold(repository.url.to_owned()),
            term::format::highlight(repository.plugins.len().to_string()),
            term::format::primary(commit_id),
            term::format::bold(date),
        ])
    }
    table.print();

    Ok(())
}

pub fn show_nurse_result(
    nurse_result: Result<CoffeeNurse, CoffeeError>,
) -> Result<(), CoffeeError> {
    match nurse_result {
        Ok(nurse) => {
            // special case: if the nurse is sane
            // we print a message and return
            if nurse.is_sane() {
                term::success!("Coffee configuration is not corrupt! No need to run coffee nurse");
                return Ok(());
            }
            let mut table = radicle_term::Table::new(TableOptions::bordered());
            table.push([
                term::format::dim(String::from("●")),
                term::format::bold(String::from("Actions Taken")),
                term::format::bold(String::from("Affected repositories")),
            ]);
            table.divider();

            for status in &nurse.status {
                let action_str = match status {
                    NurseStatus::RepositoryLocallyRestored(_) => "Restored using Git".to_string(),
                    NurseStatus::RepositoryLocallyRemoved(_) => {
                        "Removed from local storage".to_string()
                    }
                };
                let repos_str = match status {
                    NurseStatus::RepositoryLocallyRestored(repos)
                    | NurseStatus::RepositoryLocallyRemoved(repos) => repos.join(", "),
                };

                table.push([
                    term::format::positive("●").into(),
                    term::format::bold(action_str.clone()),
                    term::format::highlight(repos_str.clone()),
                ]);
            }

            table.print();
        }
        Err(err) => eprintln!("{}", err),
    }
    Ok(())
}

pub fn show_tips(coffee_tip: &CoffeeTip) -> Result<(), CoffeeError> {
    term::println(term::format::bold("●"), term::format::tertiary("Plugin"));
    let mut table = radicle_term::Table::new(TableOptions::bordered());
    table.push([
        term::format::dim(String::from("●")),
        term::format::bold(String::from("Plugin")),
        term::format::bold(String::from("Receiver")),
        term::format::bold(String::from("Amount Sent (msat)")),
    ]);
    table.divider();

    table.push([
        if coffee_tip.status == "completed" {
            term::format::positive("●").into()
        } else {
            term::format::negative("●").into()
        },
        term::format::highlight(coffee_tip.for_plugin.clone()),
        term::format::bold(coffee_tip.destination.clone().unwrap_or_default()),
        term::format::highlight(coffee_tip.amount_msat.to_string()),
    ]);
    table.print();
    Ok(())
}
