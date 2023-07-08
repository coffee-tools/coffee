#[cfg(test)]
mod coffee_httpd_integration_tests;
#[cfg(test)]
mod coffee_integration_tests;
#[cfg(test)]
mod coffee_plugin_integration_tests;
#[cfg(test)]
pub(crate) mod logger;

#[cfg(test)]
use std::sync::Once;

#[cfg(test)]
static INIT: Once = Once::new();

#[cfg(test)]
fn init() {
    // ignore error
    INIT.call_once(|| {
        logger::init(log::Level::Debug).expect("initializing logger for the first time");
    });
}
