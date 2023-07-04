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
        CoffeeError {
            code,
            msg: msg.to_string(),
        }
    }
}

impl std::error::Error for CoffeeError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(self)
    }

    fn description(&self) -> &str {
        &self.msg
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
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

impl From<String> for CoffeeError {
    fn from(value: String) -> Self {
        CoffeeError {
            code: 1,
            msg: value,
        }
    }
}
