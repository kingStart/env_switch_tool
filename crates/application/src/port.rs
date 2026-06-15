use envtools_domain::error::DomainError;
use envtools_domain::service::group_policy::ResolvedEnvironment;

/// Port: writes the resolved environment snapshot to a file that shell hooks can source.
pub trait StateFileWriter: Send + Sync {
    fn write_bash(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;
    fn write_powershell(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;
    fn write_fish(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;
}
