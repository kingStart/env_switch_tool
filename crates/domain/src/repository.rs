use crate::error::DomainError;
use crate::model::env_group::EnvGroup;

/// Port: persistence abstraction for EnvGroup aggregate.
pub trait GroupRepository: Send + Sync {
    fn find_by_name(&self, name: &str) -> Result<Option<EnvGroup>, DomainError>;
    fn find_all(&self) -> Result<Vec<EnvGroup>, DomainError>;
    fn find_active(&self) -> Result<Vec<EnvGroup>, DomainError>;
    fn save(&self, group: &EnvGroup) -> Result<(), DomainError>;
    fn delete(&self, name: &str) -> Result<(), DomainError>;
    fn exists(&self, name: &str) -> Result<bool, DomainError>;
}

/// Port: system-level environment variable operations.
pub trait SystemEnvRepository: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, DomainError>;
    fn set(&self, key: &str, value: &str) -> Result<(), DomainError>;
    fn remove(&self, key: &str) -> Result<(), DomainError>;
    /// Notify the OS that environment has changed (e.g. WM_SETTINGCHANGE on Windows).
    fn broadcast_change(&self) -> Result<(), DomainError>;
}
