use log::error;
use std::fmt;

/// Specific repository error.
#[derive(Debug, Clone)]
pub struct CoffeeError {
    code: u64,
    msg: String,
}

impl CoffeeError {
    /// Build a new error message with a specific code
    /// and a specific message.
    pub fn new(code: u64, msg: &str) -> Self {
        error!("ERROR #{}: {}", code, msg);
        CoffeeError {
            code,
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for CoffeeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}, msg: {}", self.code, self.msg)
    }
}

impl From<std::io::Error> for CoffeeError {
    fn from(err: std::io::Error) -> Self {
        CoffeeError {
            code: 1,
            msg: format!("{}", err),
        }
    }
}
