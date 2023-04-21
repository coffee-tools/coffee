//! Implementing the code function to show
//! the command result on the terminal!
use serde_json::Value;


pub fn show_remotes(remotes: Result<Value>) -> Result<(), CoffeeError> {
    let remotes = remotes?;
    radicle_term::format::bold(serde_json::to_string_pretty(remotes).unwrap());
    Ok(())
}
