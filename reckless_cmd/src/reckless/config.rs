//! Reckless configuration utils.

use std::env;

use reckless_lib::errors::RecklessError;

use super::cmd::RecklessArgs;

/// Custom reckless configuration, given by a command line list of arguments
/// or a reckless configuration file.
pub struct RecklessConf {
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

impl RecklessConf {
    /// Create a new instance of the reckless configuration from the args.
    pub async fn new(conf: &RecklessArgs) -> Result<Self, RecklessError> {
        let mut def_path = env::home_dir().unwrap().to_str().unwrap().to_string();
        // FIXME: check for double slash
        def_path += ".coffe";
        let mut reckless = RecklessConf {
            network: "bitcoin".to_owned(),
            root_path: format!("{def_path}/"),
            config: format!("{def_path}/bitcoin/coffe.conf"),
            plugins_path: vec![],
        };

        // check the command line arguments and bind them
        // inside the reckles conf
        reckless.bind_cmd_line_params(&conf)?;
        // after we know all the information regarding
        // the configuration we try to see if there is
        // something stored already to the disk.
        reckless.load_from_file().await?;

        Ok(reckless)
    }

    async fn load_from_file(&mut self) -> Result<(), RecklessError> {
        Ok(())
    }

    fn bind_cmd_line_params(&mut self, conf: &RecklessArgs) -> Result<(), RecklessError> {
        if let Some(network) = &conf.network {
            self.network = network.to_owned();
            self.config = format!("{}/{}/coffe.conf", self.root_path, self.network);
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
