use envtools_domain::error::DomainError;
use envtools_domain::repository::GroupRepository;
use envtools_domain::service::group_policy::GroupPolicy;

use crate::port::StateFileWriter;

/// Recomputes the active environment and writes state files for shell hooks.
pub struct SyncEnvironmentUseCase<'a> {
    repo: &'a dyn GroupRepository,
    state_writer: &'a dyn StateFileWriter,
}

impl<'a> SyncEnvironmentUseCase<'a> {
    pub fn new(repo: &'a dyn GroupRepository, state_writer: &'a dyn StateFileWriter) -> Self {
        Self { repo, state_writer }
    }

    pub fn execute(&self) -> Result<(), DomainError> {
        let active_groups = self.repo.find_active()?;
        let refs: Vec<&_> = active_groups.iter().collect();

        let separator = if cfg!(windows) { ";" } else { ":" };
        let resolved = GroupPolicy::resolve(&refs, separator);

        self.state_writer.write_bash(&resolved)?;
        self.state_writer.write_powershell(&resolved)?;
        self.state_writer.write_fish(&resolved)?;

        Ok(())
    }
}
