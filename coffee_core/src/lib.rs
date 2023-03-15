pub mod coffee;
pub mod config;

#[derive(Clone, Debug)]
pub enum CoffeeOperation {
    /// Install(plugin name, verbose run, dynamic installation)
    Install(String, bool, bool),
    /// List(include remotes)
    List(bool),
    Upgrade,
    Remove,
    /// Remote(name repository, url of the repositoryu)
    Remote(RemoteAction),
    /// Setup(core lightning root path)
    Setup(String),
}

#[derive(Clone, Debug)]
pub enum RemoteAction {
    Add(String, String),
    Remove(String),
}

pub trait CoffeeArgs {
    /// returnt the command that coffee need to execute
    fn command(&self) -> CoffeeOperation;
    /// return the conf
    fn conf(&self) -> Option<String>;
    /// return the network
    fn network(&self) -> Option<String>;
    /// return the data dir
    fn data_dir(&self) -> Option<String>;
}
