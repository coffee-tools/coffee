pub mod coffee;
pub mod config;

mod nurse;

pub use coffee_lib as lib;

#[derive(Clone, Debug)]
pub enum CoffeeOperation {
    /// Link coffee to the lightning configuration file
    Link(String),
    /// Unlink coffee from the lightning configuration file
    Unlink(String),
    /// Install(plugin name, verbose run, dynamic installation)
    Install(String, bool, bool),
    /// List
    List,
    // Upgrade(name of the repository, verbose run)
    Upgrade(String, bool),
    Remove(String),
    /// Remote(name repository, url of the repository)
    Remote(Option<RemoteAction>, Option<String>),
    Show(String),
    /// Search(plugin name)
    Search(String),
    Nurse(bool),
    /// Tip operation
    ///
    /// (plugin_name, amount_msat)
    Tip(String, u64),
    /// Disable a plugin(plugin name)
    Disable(String),
    /// Enable a plugin(plugin name)
    Enable(String),
}

#[derive(Clone, Debug)]
pub enum RemoteAction {
    Add(String, String),
    Rm(String),
    Inspect(String),
    List,
}

pub trait CoffeeArgs: Send + Sync {
    /// return the command that coffee needs to execute
    fn command(&self) -> CoffeeOperation;
    /// return the conf
    fn conf(&self) -> Option<String>;
    /// return the network
    fn network(&self) -> Option<String>;
    /// return the data dir
    fn data_dir(&self) -> Option<String>;
    /// return the skip verify flag
    fn skip_verify(&self) -> bool;
}
