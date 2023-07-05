//! Coffee Model Definition
use serde::{Deserialize, Serialize};

use crate::plugin::Plugin;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoffeeRemove {
    pub plugin: Plugin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoffeeList {
    pub plugins: Vec<Plugin>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoffeeRemote {
    pub remotes: Option<Vec<CoffeeListRemote>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoffeeListRemote {
    pub local_name: String,
    pub url: String,
    pub plugins: Vec<Plugin>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NurseStatus {
    Corrupted,
    Sane,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoffeeNurse {
    pub status: NurseStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UpgradeStatus {
    UpToDate,
    Updated,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoffeeUpgrade {
    pub repo: String,
    pub status: UpgradeStatus,
    /// If the status of the repository is
    /// alterate we return the list of plugin
    /// that are effected and need to be recompiled.
    pub plugins_effected: Vec<String>,
}
