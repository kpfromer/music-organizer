use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required env var {var} is not set or is empty")]
    Missing { var: &'static str },

    #[error("env var {var} has an invalid value: {reason}")]
    Invalid { var: &'static str, reason: String },
}
