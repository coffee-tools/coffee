//! Coffee Model Definition
use serde::{Deserialize, Serialize};

use crate::plugin::Plugin;

#[derive(Serialize, Deserialize)]
pub struct CoffeeRemove {
    pub plugin: Plugin,
}

#[derive(Serialize, Deserialize)]
pub struct CoffeeList {
    pub remotes: Option<Vec<CoffeeListRemote>>,
    pub plugins: Vec<Plugin>,
}

#[derive(Serialize, Deserialize)]
pub struct CoffeeRemote {
    pub remotes: Option<Vec<CoffeeListRemote>>,
}

#[derive(Serialize, Deserialize)]
pub struct CoffeeListRemote {
    pub local_name: String,
    pub url: String,
    pub plugins: Vec<Plugin>,
}
