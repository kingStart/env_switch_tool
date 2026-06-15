use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("group '{0}' not found")]
    GroupNotFound(String),

    #[error("group '{0}' already exists")]
    GroupAlreadyExists(String),

    #[error("variable key cannot be empty")]
    EmptyVariableKey,

    #[error("invalid variable key '{0}': must be [A-Za-z_][A-Za-z0-9_]*")]
    InvalidVariableKey(String),

    #[error("circular variable reference detected: {0}")]
    CircularReference(String),
}
