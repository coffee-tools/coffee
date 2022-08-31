mod reckless;

use crate::reckless::cmd::RecklessArgs;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args = RecklessArgs::parse();
    Ok(())
}
