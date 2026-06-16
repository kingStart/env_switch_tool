pub mod env_variable;
pub mod group_kind;
pub mod hosts_entry;
pub mod managed_group;
pub mod priority;
pub mod profile;

/// Backward-compatible alias
pub mod env_group {
    pub use super::managed_group::ManagedGroup as EnvGroup;
}
