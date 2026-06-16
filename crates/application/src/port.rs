use envtools_domain::error::DomainError;
use envtools_domain::model::hosts_entry::HostsEntry;
use envtools_domain::service::group_policy::ResolvedEnvironment;

/// Port: writes the resolved environment snapshot to a file that shell hooks can source.
pub trait StateFileWriter: Send + Sync {
    fn write_bash(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;
    fn write_powershell(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;
    fn write_fish(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError>;

    fn write_system_env(&self, _resolved: &ResolvedEnvironment) -> Result<(), DomainError> {
        Ok(())
    }
}

/// Port: writes hosts entries to the system hosts file using marker blocks.
pub trait HostsFileWriter: Send + Sync {
    fn apply_hosts(&self, entries: &[HostsEntry]) -> Result<(), DomainError>;
    fn clear_managed(&self) -> Result<(), DomainError>;
}

/// Port: runs a CLI command with elevated privileges.
pub trait ElevationService: Send + Sync {
    fn run_elevated(&self, args: &[&str]) -> Result<(), DomainError>;
}
