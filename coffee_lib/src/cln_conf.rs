//! Manage core lightning configuration
use std::fmt::Display;

use log::debug;

use crate::plugin::Plugin;

pub struct CLNConf {
    pub path: String,
    pub network: String,
    pub plugins: Vec<Plugin>,
}

impl CLNConf {
    pub fn new(network: &str, path: &str) -> Self {
        CLNConf {
            path: path.to_owned(),
            network: network.to_string(),
            plugins: vec![],
        }
    }
}

impl Display for CLNConf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut conf_str = "# coffee configuration\n".to_owned();
        for plugin in &self.plugins {
            conf_str += format!("plugin={}\n", plugin.path).as_str();
        }
        debug!("store the following cln conf");
        debug!("{conf_str}");
        write!(f, "{conf_str}")
    }
}
