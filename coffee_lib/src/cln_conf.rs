//! Manage core lightning configuration
use crate::plugin::Plugin;

pub struct CLNConf {
    pub network: String,
    pub plugins: Vec<Plugin>,
}

impl CLNConf {
    pub fn new(network: &str) -> Self {
        CLNConf {
            network: network.to_string(),
            plugins: vec![],
        }
    }
}
