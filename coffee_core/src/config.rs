//! Coffee configuration utils.
use log::info;
use serde::{Deserialize, Serialize};
use std::env;

use crate::CoffeeOperation;
use coffee_lib::utils::{check_dir_or_make_if_missing, copy_dir_if_exist};
use coffee_lib::{errors::CoffeeError, plugin::Plugin};

use crate::CoffeeArgs;
/// Custom coffee configuration, given by a command line list of arguments
/// or a coffee configuration file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoffeeConf {
    /// Network configuration related
    /// to core lightning network
    pub network: String,
    /// path of core lightning configuration file
    /// managed by coffee
    pub config_path: String,
    /// path of the core lightning configuration file
    /// not managed by core lightning
    /// (this file included the file managed by coffee)
    pub cln_config_path: Option<String>,
    /// root cln directory path
    pub cln_root: Option<String>,
    /// root path plugin manager
    pub root_path: String,
    /// all plugins that are installed
    /// with the plugin manager.
    pub plugins: Vec<Plugin>,
    /// A flag that indicates if the
    /// user wants to skip the verification
    /// of nurse.
    pub skip_verify: bool,
}

impl CoffeeConf {
    /// Create a new instance of the coffee configuration from the args.
    pub async fn new(conf: &dyn CoffeeArgs) -> Result<Self, CoffeeError> {
        #[allow(deprecated)]
        let mut def_path = env::home_dir().unwrap().to_str().unwrap().to_string();
        if let Some(data_dir) = &conf.data_dir() {
            def_path = data_dir.to_owned();
        }

        def_path = def_path.strip_suffix('/').unwrap_or(&def_path).to_string();
        def_path += "/.coffee";
        check_dir_or_make_if_missing(def_path.to_string()).await?;
        info!("creating coffee home at {def_path}");

        let mut coffee = CoffeeConf {
            network: "bitcoin".to_owned(),
            root_path: def_path.to_string(),
            config_path: format!("{def_path}/bitcoin/coffee.conf"),
            plugins: vec![],
            cln_config_path: None,
            cln_root: None,
            skip_verify: false,
        };

        // check the command line arguments and bind them
        // inside the coffee conf
        coffee.bind_cmd_line_params(conf)?;

        check_dir_or_make_if_missing(format!("{def_path}/{}", coffee.network)).await?;
        check_dir_or_make_if_missing(format!("{def_path}/{}/plugins", coffee.network)).await?;
        let repo_dir = format!("{def_path}/{}/repositories", coffee.network);
        // older version of coffee has a repository inside the directory
        copy_dir_if_exist(&format!("{def_path}/repositories"), &repo_dir).await?;
        // FIXME: nurse should clean up the  `{def_path}/repositories`.
        check_dir_or_make_if_missing(repo_dir).await?;
        // after we know all the information regarding
        // the configuration we try to see if there is
        // something stored already to the disk.
        coffee.load_from_file().await?;

        Ok(coffee)
    }

    async fn load_from_file(&mut self) -> Result<(), CoffeeError> {
        Ok(())
    }

    fn bind_cmd_line_params(&mut self, conf: &dyn CoffeeArgs) -> Result<(), CoffeeError> {
        if let Some(network) = &conf.network() {
            self.network = network.to_owned();
            self.config_path = format!("{}/{}/coffee.conf", self.root_path, self.network);
        }

        if let Some(config) = &conf.conf() {
            self.config_path = config.to_owned();
        }

        // If the command is nurse we skip the verification
        // because nurse is the command that needs
        // to solve the configuration problems.
        if !conf.skip_verify() {
            match conf.command() {
                CoffeeOperation::Nurse(_) => {
                    self.skip_verify = true;
                }
                _ => {
                    self.skip_verify = false;
                }
            }
        }

        // FIXME: be able to put the directory also in another place!
        // for now it is fixed in the Home/.coffee but another good place
        // will be, the .lightning dir
        Ok(())
    }
}
