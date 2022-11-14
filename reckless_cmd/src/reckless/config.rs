//! Reckless configuration utils.

use reckless_lib::errors::RecklessError;

use super::cmd::RecklessArgs;

/// Custom reckless configuration, given by a command line list of arguments
/// or a reckless configuration file.
pub struct RecklessConf {
    network: Option<String>,
}

impl RecklessConf {
    /// Create a new instance of the reckless configuration from the args.
    pub async fn new(conf: &RecklessArgs) -> Result<Self, RecklessError> {
        let mut reckless = RecklessConf { network: None };

        if let Some(conf_path) = conf.conf.to_owned() {
            reckless.load_from_file(&conf_path.as_str()).await?;
        }
        reckless.bind_cmd_line_params(&conf)?;
        Ok(reckless)
    }

    async fn load_from_file(&mut self, conf_path: &str) -> Result<(), RecklessError> {
        Ok(())
    }

    fn bind_cmd_line_params(&mut self, conf: &RecklessArgs) -> Result<(), RecklessError> {
        if let Some(network) = conf.network.to_owned() {
            self.network = Some(network);
        }

        Ok(())
    }
}
