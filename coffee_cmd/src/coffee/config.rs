//! Coffee configuration utils.

use std::env;

use coffee_lib::errors::CoffeeError;

use super::cmd::CoffeeArgs;

/// Custom coffee configuration, given by a command line list of arguments
/// or a coffee configuration file.
pub struct CoffeeConf {
    /// Network configuration related
    /// to core lightning network
    network: String,
    /// plugin manager configuration path
    config: String,
    /// root path plugin manager
    pub root_path: String,
    /// path of all plugin that are installed
    /// with the plugin manager.
    pub plugins_path: Vec<String>,
}

impl CoffeeConf {
    /// Create a new instance of the coffee configuration from the args.
    pub async fn new(conf: &CoffeeArgs) -> Result<Self, CoffeeError> {
        let mut def_path = env::home_dir().unwrap().to_str().unwrap().to_string();
        // FIXME: check for double slash
        def_path += "/.coffee";
        let mut coffee = CoffeeConf {
            network: "bitcoin".to_owned(),
            root_path: format!("{def_path}"),
            config: format!("{def_path}/bitcoin/coffee.conf"),
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
            self.config = format!("{}/{}/coffee.conf", self.root_path, self.network);
        }

        if let Some(config) = &conf.conf {
            self.config = config.to_owned();
        }

        // FIXME: be able to put the directory also in another place!
        // for now it is fixed in the Home/.coffe but another good place
        // will be, the .lightning dir
        Ok(())
    }
}
