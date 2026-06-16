use crate::error::DomainError;
use crate::model::managed_group::ManagedGroup;
use crate::model::profile::Profile;

/// Port: persistence abstraction for ManagedGroup aggregate.
pub trait GroupRepository: Send + Sync {
    fn find_by_name(&self, name: &str) -> Result<Option<ManagedGroup>, DomainError>;
    fn find_all(&self) -> Result<Vec<ManagedGroup>, DomainError>;
    fn find_active(&self) -> Result<Vec<ManagedGroup>, DomainError>;
    fn save(&self, group: &ManagedGroup) -> Result<(), DomainError>;
    fn delete(&self, name: &str) -> Result<(), DomainError>;
    fn exists(&self, name: &str) -> Result<bool, DomainError>;
}

/// Port: persistence abstraction for Profile aggregate.
pub trait ProfileRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<Profile>, DomainError>;
    fn find_by_name(&self, name: &str) -> Result<Option<Profile>, DomainError>;
    fn save(&self, profile: &Profile) -> Result<(), DomainError>;
    fn delete(&self, name: &str) -> Result<(), DomainError>;
    fn exists(&self, name: &str) -> Result<bool, DomainError>;
}

/// Port: system-level environment variable operations.
pub trait SystemEnvRepository: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, DomainError>;
    fn set(&self, key: &str, value: &str) -> Result<(), DomainError>;
    fn remove(&self, key: &str) -> Result<(), DomainError>;
    fn broadcast_change(&self) -> Result<(), DomainError>;
}
