//! Reckless command line arguments definition.
use clap::{Parser, Subcommand};

/// Reckless main command line definition for the command line tools.
#[derive(Debug, Parser)]
#[clap(name = "rkl")]
#[clap(about = "A reckless plugin manager for core lightning", long_about = None)]
pub struct RecklessArgs {
    #[clap(subcommand)]
    pub command: RecklessCommand,
    #[clap(short, long, value_parser)]
    pub conf: Option<String>,
    #[clap(short, long, value_parser)]
    pub network: Option<String>,
}

/// Reckless subcommand of the command line daemon.
#[derive(Debug, Subcommand)]
pub enum RecklessCommand {
    /// Install a single or a list of plugins.
    #[clap(arg_required_else_help = true)]
    Install,
    /// upgrade a single or a list of plugins.
    #[clap(arg_required_else_help = true)]
    Upgrade,
    /// Print the list of plugin installed in cln.
    #[clap(arg_required_else_help = true)]
    List,
    /// Remove a plugin installed in cln.
    #[clap(arg_required_else_help = true)]
    Remove,
}
