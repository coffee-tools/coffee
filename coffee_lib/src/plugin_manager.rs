//! Plugin manager module definition.
use async_trait::async_trait;

use crate::{errors::CoffeeError, types::response::*};

/// Plugin manager traits that define the API a generic
/// plugin manager.
#[async_trait]
pub trait PluginManager {
    /// configure the plugin manger.
    async fn configure(&mut self) -> Result<(), CoffeeError>;

    /// install a plugin by name, return an error if some error happens.
    async fn install(
        &mut self,
        plugins: &str,
        verbose: bool,
        try_dynamic: bool,
    ) -> Result<(), CoffeeError>;

    // remove a plugin by name, return an error if some error happens.
    async fn remove(&mut self, plugin: &str) -> Result<CoffeeRemove, CoffeeError>;

    /// return the list of plugins installed by the plugin manager.
    async fn list(&mut self) -> Result<CoffeeList, CoffeeError>;

    /// upgrade a single or multiple repositories.
    async fn upgrade(&mut self, repo: &str, verbose: bool) -> Result<CoffeeUpgrade, CoffeeError>;

    /// add the remote repository to the plugin manager.
    async fn add_remote(&mut self, name: &str, url: &str) -> Result<(), CoffeeError>;

    /// remove the remote repository from the plugin manager.
    async fn rm_remote(&mut self, name: &str) -> Result<(), CoffeeError>;

    /// list the remote repositories for the plugin manager.
    async fn list_remotes(&mut self) -> Result<CoffeeRemote, CoffeeError>;

    /// List the plugins available in a remote repository.
    async fn get_plugins_in_remote(&self, name: &str) -> Result<CoffeeList, CoffeeError>;

    /// set up the core lightning configuration target for the
    /// plugin manager.
    async fn setup(&mut self, cln_conf_path: &str) -> Result<(), CoffeeError>;

    /// show the README file of the plugin
    async fn show(&mut self, plugin: &str) -> Result<CoffeeShow, CoffeeError>;

    /// search remote repositories for a plugin by name
    async fn search(&mut self, plugin: &str) -> Result<CoffeeSearch, CoffeeError>;

    /// clean up storage information about the remote repositories of the plugin manager.
    async fn nurse(&mut self) -> Result<CoffeeNurse, CoffeeError>;

    /// verify that coffee configuration is sane without taking any action.
    async fn nurse_verify(&self) -> Result<ChainOfResponsibilityStatus, CoffeeError>;

    /// patch coffee configuration in the case that a repository is present in the coffee
    /// configuration but is absent from the local storage.
    async fn patch_repository_locally_absent(
        &mut self,
        repos: Vec<String>,
    ) -> Result<Vec<NurseStatus>, CoffeeError>;

    /// tip a specific plugins of the following amount
    ///
    /// The tip command required that the receiver of the
    /// donation is runing the coffee core lightning plugin.
    ///
    /// P.S: only Bitcoin ofc
    async fn tip(&mut self, plugin: &str, amount_msat: u64) -> Result<CoffeeTip, CoffeeError>;

    /// disable a plugin by name
    async fn disable(&mut self, plugin: &str) -> Result<(), CoffeeError>;

    /// enable a plugin by name
    async fn enable(&mut self, plugin: &str) -> Result<(), CoffeeError>;

    /// The repositories home directory, where all the dirs
    /// will be copied and cloned
    fn repositories_home(&self) -> String;

    /// The plugins home directory
    ///
    /// When we install a plugin we will have to copy it in
    /// another plugin directory and this is the home
    /// (aka root path for it).
    fn plugins_home(&self) -> String;
}
