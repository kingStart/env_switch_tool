use envtools_domain::error::DomainError;
use envtools_domain::service::group_policy::ResolvedEnvironment;

/// Port: writes the resolved environment snapshot to a file that shell hooks can source.
pub trait StateFileWriter: Send + Sync {
    fn write_bash(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;
    fn write_powershell(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;
    fn write_fish(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;

    /// Write to system-level environment (Windows Registry, etc.) for CMD/non-hook shells.
    /// Default no-op for platforms without system env integration.
    fn write_system_env(&self, _resolved: &ResolvedEnvironment) -> Result<(), DomainError> {
        Ok(())
    }
}
