//! Minimal information to make
//! a repository persistent.
use coffee_lib::{plugin::Plugin, url::URL};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Kind {
    Git,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub kind: Kind,
    pub name: String,
    pub url: URL,
    pub plugins: Vec<Plugin>,
    pub branch: String,
}
