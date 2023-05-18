//! Minimal information to make persistent
//! a repository.
use coffee_lib::{plugin::Plugin, url::URL};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Kind {
    Git,
}

#[derive(Serialize, Deserialize)]
pub struct Repository {
    pub kind: Kind,
    pub name: String,
    pub url: URL,
    pub plugins: Vec<Plugin>,
    pub branch: String,
}
