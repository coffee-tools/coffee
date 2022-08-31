//! Reckless command line arguments!
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "rkl")]
#[clap(about = "A reckless plugin manager for core lightning", long_about = None)]
pub struct RecklessArgs {
    #[clap(subcommand)]
    command: RecklessCommand,
    #[clap(short, long, value_parser)]
    conf: Option<String>,
}

#[derive(Debug, Subcommand)]
enum RecklessCommand {
    #[clap(arg_required_else_help = true)]
    Install,
}
