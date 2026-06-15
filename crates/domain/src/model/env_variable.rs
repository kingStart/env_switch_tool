use crate::error::DomainError;
use std::fmt;

/// How a variable interacts with existing PATH-like values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathMode {
    /// Replace the value entirely (default for non-PATH vars).
    Override,
    /// Prepend to the existing value with OS separator.
    Prepend,
    /// Append to the existing value with OS separator.
    Append,
}

/// Value Object representing a single environment variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVariable {
    key: String,
    value: String,
    path_mode: PathMode,
}

impl EnvVariable {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, DomainError> {
        let key = key.into();
        Self::validate_key(&key)?;
        Ok(Self {
            key,
            value: value.into(),
            path_mode: PathMode::Override,
        })
    }

    pub fn with_path_mode(
        key: impl Into<String>,
        value: impl Into<String>,
        mode: PathMode,
    ) -> Result<Self, DomainError> {
        let key = key.into();
        Self::validate_key(&key)?;
        Ok(Self {
            key,
            value: value.into(),
            path_mode: mode,
        })
    }

    fn validate_key(key: &str) -> Result<(), DomainError> {
        if key.is_empty() {
            return Err(DomainError::EmptyVariableKey);
        }
        let first = key.chars().next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(DomainError::InvalidVariableKey(key.to_string()));
        }
        if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(DomainError::InvalidVariableKey(key.to_string()));
        }
        Ok(())
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn path_mode(&self) -> &PathMode {
        &self.path_mode
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}

impl fmt::Display for EnvVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}
