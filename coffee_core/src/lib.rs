pub mod coffee;
pub mod config;

#[derive(Clone, Debug)]
pub enum CoffeeOperation {
    /// Install(plugin name, verbose run, dynamic installation)
    Install(String, bool, bool),
    /// List
    List,
    // Upgrade(name of the repository)
    Upgrade(String),
    Remove(String),
    /// Remote(name repository, url of the repository)
    Remote(RemoteAction),
    /// Setup(core lightning root path)
    Setup(String),
    Show(String),
    Nurse,
}

#[derive(Clone, Debug)]
pub enum RemoteAction {
    Add(String, String),
    Rm(String),
    List,
}

pub trait CoffeeArgs {
    /// return the command that coffee needs to execute
    fn command(&self) -> CoffeeOperation;
    /// return the conf
    fn conf(&self) -> Option<String>;
    /// return the network
    fn network(&self) -> Option<String>;
    /// return the data dir
    fn data_dir(&self) -> Option<String>;
}
