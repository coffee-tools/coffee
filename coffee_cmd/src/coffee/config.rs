//! Coffee configuration utils.

use coffee_lib::errors::CoffeeError;
use serde::{Deserialize, Serialize};
use std::{env, path::Path};
use tokio::fs::create_dir;

use super::cmd::CoffeeArgs;

/// Custom coffee configuration, given by a command line list of arguments
/// or a coffee configuration file.
#[derive(Clone, Serialize, Deserialize)]
pub struct CoffeeConf {
    /// Network configuration related
    /// to core lightning network
    pub network: String,
    /// core lightning configuration file path
    pub cln_config_path: String,
    /// root path plugin manager
    pub root_path: String,
    /// path of all plugin that are installed
    /// with the plugin manager.
    pub plugins_path: Vec<String>,
}

async fn check_dir_or_make_if_missing(path: String) -> Result<(), CoffeeError> {
    if !Path::exists(Path::new(&path.to_owned())) {
        create_dir(path).await?;
    }
    Ok(())
}

impl CoffeeConf {
    /// Create a new instance of the coffee configuration from the args.
    pub async fn new(conf: &CoffeeArgs) -> Result<Self, CoffeeError> {
        #[allow(deprecated)]
        let mut def_path = env::home_dir().unwrap().to_str().unwrap().to_string();
        // FIXME: check for double slash
        def_path += "/.coffee";
        check_dir_or_make_if_missing(def_path.to_string()).await?;
        check_dir_or_make_if_missing(format!("{def_path}/bitcoin")).await?;
        check_dir_or_make_if_missing(format!("{def_path}/testnet")).await?;
        let mut coffee = CoffeeConf {
            network: "bitcoin".to_owned(),
            root_path: format!("{def_path}"),
            cln_config_path: format!("{def_path}/bitcoin/coffee.conf"),
            plugins_path: vec![],
        };

        // check the command line arguments and bind them
        // inside the coffee conf
        coffee.bind_cmd_line_params(&conf)?;
        // after we know all the information regarding
        // the configuration we try to see if there is
        // something stored already to the disk.
        coffee.load_from_file().await?;

        Ok(coffee)
    }

    async fn load_from_file(&mut self) -> Result<(), CoffeeError> {
        Ok(())
    }

    fn bind_cmd_line_params(&mut self, conf: &CoffeeArgs) -> Result<(), CoffeeError> {
        if let Some(network) = &conf.network {
            self.network = network.to_owned();
            self.cln_config_path = format!("{}/{}/coffee.conf", self.root_path, self.network);
        }

        if let Some(config) = &conf.conf {
            self.cln_config_path = config.to_owned();
        }

        // FIXME: be able to put the directory also in another place!
        // for now it is fixed in the Home/.coffe but another good place
        // will be, the .lightning dir
        Ok(())
    }
}
