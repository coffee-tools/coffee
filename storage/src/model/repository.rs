//! Minimal information to make persistent
//! a repository.
use coffee_lib::plugin::Plugin;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub plugins: Vec<Plugin>,
}
