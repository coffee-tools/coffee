//! Reckless mod implementation
use std::vec::Vec;

pub mod cmd;
mod config;

pub struct RecklessManager {
    config: config::Configuration,
    repos: Vec<String>,
    plugins: Vec<String>,
}
