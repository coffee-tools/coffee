use std::fmt;

/// Specific repository error.
#[derive(Debug, Clone)]
pub struct RecklessError {
    code: u64,
    msg: String,
}

impl RecklessError {
    /// Build a new error message with a specific code
    /// and a specific message.
    fn new(code: u64, msg: &str) -> Self {
        RecklessError {
            code,
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for RecklessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}, msg: {}", self.code, self.msg)
    }
}
