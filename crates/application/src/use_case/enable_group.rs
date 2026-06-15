use envtools_domain::error::DomainError;
use envtools_domain::repository::GroupRepository;

use crate::use_case::sync_environment::SyncEnvironmentUseCase;
use crate::port::StateFileWriter;

pub struct EnableGroupUseCase<'a> {
    repo: &'a dyn GroupRepository,
    state_writer: &'a dyn StateFileWriter,
}

impl<'a> EnableGroupUseCase<'a> {
    pub fn new(repo: &'a dyn GroupRepository, state_writer: &'a dyn StateFileWriter) -> Self {
        Self { repo, state_writer }
    }

    pub fn execute(&self, name: &str) -> Result<(), DomainError> {
        let mut group = self
            .repo
            .find_by_name(name)?
            .ok_or_else(|| DomainError::GroupNotFound(name.to_string()))?;

        group.enable();
        self.repo.save(&group)?;

        SyncEnvironmentUseCase::new(self.repo, self.state_writer).execute()
    }
}
