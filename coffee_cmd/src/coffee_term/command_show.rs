//! Implementing the code function to show
//! the command result on the terminal!

use radicle_term as term;
use radicle_term::table::TableOptions;

use coffee_lib::error;
use coffee_lib::errors::CoffeeError;
use coffee_lib::types::response::{CoffeeList, CoffeeRemote};
use term::Element;

pub fn show_list(coffee_list: Result<CoffeeList, CoffeeError>) -> Result<(), CoffeeError> {
    let remotes = coffee_list?;

    term::println(
        term::format::tertiary_bold("●"),
        term::format::tertiary("Plugin installed"),
    );
    let mut table = radicle_term::Table::new(TableOptions::bordered());
    table.push([
        term::format::dim(String::from("●")),
        term::format::bold(String::from("Language")),
        term::format::bold(String::from("Name")),
        term::format::bold(String::from("Exec path")),
    ]);
    table.divider();

    for plugin in &remotes.plugins {
        table.push([
            term::format::positive("●").into(),
            term::format::highlight(plugin.lang.to_string()),
            term::format::bold(plugin.name()),
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
        term::format::tertiary_bold("●"),
        term::format::tertiary("List of repositories"),
    );
    let mut table = radicle_term::Table::new(TableOptions::bordered());
    table.push([
        term::format::dim(String::from("●")),
        term::format::bold(String::from("Repository Alias")),
        term::format::bold(String::from("URL")),
        term::format::bold(String::from("N. Plugins")),
    ]);
    table.divider();

    for repository in &repositories {
        table.push([
            term::format::positive("●").into(),
            term::format::highlight(repository.local_name.to_owned()),
            term::format::bold(repository.url.to_owned()),
            term::format::highlight(repository.plugins.len().to_string()),
        ])
    }
    table.print();

    Ok(())
}
