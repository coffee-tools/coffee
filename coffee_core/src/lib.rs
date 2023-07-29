pub mod coffee;
pub mod config;

mod nurse;

pub use coffee_lib as lib;

#[derive(Clone, Debug)]
pub enum CoffeeOperation {
    /// Install(plugin name, verbose run, dynamic installation)
    Install(String, bool, bool),
    /// List
    List,
    // Upgrade(name of the repository, verbose run)
    Upgrade(String, bool),
    Remove(String),
    /// Remote(name repository, url of the repository)
    Remote(Option<RemoteAction>, bool, Option<String>),
    /// Setup(core lightning root path)
    Setup(String),
    Show(String),
    /// Search(plugin name)
    Search(String),
    Nurse(bool),
    /// Tip operation
    ///
    /// (plugin_name, amount_msat)
    Tip(String, u64),
}

#[derive(Clone, Debug)]
pub enum RemoteAction {
    Add(String, String),
    Rm(String),
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
