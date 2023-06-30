//! Coffee command line arguments definition.
use clap::{Parser, Subcommand};

/// Coffee main command line definition for the command line tools.
#[derive(Debug, Parser)]
#[clap(name = "coffee")]
#[clap(about = "A plugin manager for core lightning", long_about = None)]
pub struct CoffeeArgs {
    #[clap(subcommand)]
    pub command: CoffeeCommand,
    #[clap(short, long, value_parser)]
    pub conf: Option<String>,
    #[clap(short, long, value_parser)]
    pub network: Option<String>,
    #[clap(short, long, value_parser, name = "data-dir")]
    pub data_dir: Option<String>,
}

/// Coffee subcommand of the command line daemon.
#[derive(Debug, Subcommand)]
pub enum CoffeeCommand {
    /// Install a single by name.
    #[clap(arg_required_else_help = true)]
    Install {
        plugin: String,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        verbose: bool,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        dynamic: bool,
    },
    /// upgrade a single repository.
    #[clap(arg_required_else_help = true)]
    Upgrade {
        repo: String,
        #[clap(short, long, value_parser, name = "branch")]
        branch: Option<String>,
    },
    /// Print the list of plugins installed in cln.
    #[clap(arg_required_else_help = false)]
    List {},
    /// Remove a plugin installed in cln.
    #[clap(arg_required_else_help = true)]
    Remove { plugin: String },
    /// Manage Repository subcommand
    #[clap(arg_required_else_help = true)]
    Remote {
        #[clap(subcommand)]
        action: RemoteAction,
    },
    /// Configure coffee with the core lightning
    /// configuration
    #[clap(arg_required_else_help = true)]
    Setup { cln_conf: String },
    /// show the README file of the plugin
    #[clap(arg_required_else_help = true)]
    Show { plugin: String },
    /// clean up remote repositories storage information
    #[clap(arg_required_else_help = false)]
    Nurse {},
}

#[derive(Debug, Subcommand)]
pub enum RemoteAction {
    Add { name: String, url: String },
    Rm { name: String },
    List {},
}

impl From<&CoffeeCommand> for coffee_core::CoffeeOperation {
    fn from(value: &CoffeeCommand) -> Self {
        match value {
            CoffeeCommand::Install {
                plugin,
                verbose,
                dynamic,
            } => Self::Install(plugin.to_owned(), *verbose, *dynamic),
            CoffeeCommand::Upgrade { repo, branch } => {
                Self::Upgrade(repo.to_owned(), branch.to_owned())
            }
            CoffeeCommand::List {} => Self::List,
            CoffeeCommand::Setup { cln_conf } => Self::Setup(cln_conf.to_owned()),
            CoffeeCommand::Remote { action } => Self::Remote(action.into()),
            CoffeeCommand::Remove { plugin } => Self::Remove(plugin.to_owned()),
            CoffeeCommand::Show { plugin } => Self::Show(plugin.to_owned()),
            CoffeeCommand::Nurse {} => Self::Nurse,
        }
    }
}

impl From<&RemoteAction> for coffee_core::RemoteAction {
    fn from(value: &RemoteAction) -> Self {
        match value {
            RemoteAction::Add { name, url } => Self::Add(name.to_owned(), url.to_owned()),
            RemoteAction::Rm { name } => Self::Rm(name.to_owned()),
            RemoteAction::List {} => Self::List,
        }
    }
}

impl coffee_core::CoffeeArgs for CoffeeArgs {
    fn command(&self) -> coffee_core::CoffeeOperation {
        coffee_core::CoffeeOperation::from(&self.command)
    }

    fn conf(&self) -> Option<String> {
        self.conf.clone()
    }

    fn data_dir(&self) -> Option<String> {
        self.data_dir.clone()
    }

    fn network(&self) -> Option<String> {
        self.network.clone()
    }
}
