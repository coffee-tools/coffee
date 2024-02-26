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
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    pub skip_verify: bool,
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
        #[arg(short, long)]
        branch: Option<String>,
    },
    /// upgrade a single repository.
    #[clap(arg_required_else_help = true)]
    Upgrade {
        repo: String,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        verbose: bool,
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
        action: Option<RemoteAction>,
        #[arg(name = "remote-name", help = "The name of the remote repository")]
        name: Option<String>,
    },
    /// Configure coffee with the core lightning
    /// configuration
    #[clap(arg_required_else_help = true)]
    Setup { cln_conf: String },
    /// show the README file of the plugin
    #[clap(arg_required_else_help = true)]
    Show { plugin: String },
    /// search the remote repositories for a plugin
    #[clap(arg_required_else_help = true)]
    Search { plugin: String },
    /// clean up remote repositories storage information
    #[clap(arg_required_else_help = false)]
    Nurse {
        /// verify that coffee configuration is sane (without taking any action)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        verify: bool,
    },
    /// tipping a plugins developer.
    #[clap(arg_required_else_help = false)]
    Tip { plugin: String, amount_msat: u64 },
    /// Disable a plugin
    #[clap(arg_required_else_help = true)]
    Disable { plugin: String },
    /// Enable a plugin
    #[clap(arg_required_else_help = true)]
    Enable { plugin: String },
}

#[derive(Debug, Subcommand)]
pub enum RemoteAction {
    /// Add a remote repository to the plugin manager.
    Add { name: String, url: String },
    /// Remove a remote repository from the plugin manager.
    Rm { name: String },
    /// Inspect the plugins available in a remote repository.
    Inspect { name: String },
    /// List the remote repositories from the plugin manager.
    List {},
}

impl From<&CoffeeCommand> for coffee_core::CoffeeOperation {
    fn from(value: &CoffeeCommand) -> Self {
        match value {
            CoffeeCommand::Install {
                plugin,
                branch,
                verbose,
                dynamic,
            } => Self::Install(plugin.to_owned(), branch.clone(), *verbose, *dynamic),
            CoffeeCommand::Upgrade { repo, verbose } => Self::Upgrade(repo.to_owned(), *verbose),
            CoffeeCommand::List {} => Self::List,
            CoffeeCommand::Setup { cln_conf } => Self::Setup(cln_conf.to_owned()),
            CoffeeCommand::Remote { action, name } => {
                if let Some(action) = action {
                    return Self::Remote(Some(action.into()), name.clone());
                }
                Self::Remote(None, name.clone())
            }
            CoffeeCommand::Remove { plugin } => Self::Remove(plugin.to_owned()),
            CoffeeCommand::Show { plugin } => Self::Show(plugin.to_owned()),
            CoffeeCommand::Search { plugin } => Self::Search(plugin.to_owned()),
            CoffeeCommand::Nurse { verify } => Self::Nurse(*verify),
            CoffeeCommand::Tip {
                plugin,
                amount_msat,
            } => Self::Tip(plugin.to_owned(), amount_msat.clone()),
            CoffeeCommand::Disable { plugin } => Self::Disable(plugin.to_owned()),
            CoffeeCommand::Enable { plugin } => Self::Enable(plugin.to_owned()),
        }
    }
}

impl From<&RemoteAction> for coffee_core::RemoteAction {
    fn from(value: &RemoteAction) -> Self {
        match value {
            RemoteAction::Add { name, url } => Self::Add(name.to_owned(), url.to_owned()),
            RemoteAction::Rm { name } => Self::Rm(name.to_owned()),
            RemoteAction::Inspect { name } => Self::Inspect(name.to_owned()),
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

    fn skip_verify(&self) -> bool {
        self.skip_verify
    }
}
