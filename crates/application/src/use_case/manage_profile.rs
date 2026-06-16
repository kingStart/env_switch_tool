use envtools_domain::error::DomainError;
use envtools_domain::model::profile::Profile;
use envtools_domain::repository::{GroupRepository, ProfileRepository};

use crate::dto::ProfileInfo;
use crate::port::StateFileWriter;
use crate::use_case::sync_environment::SyncEnvironmentUseCase;

pub struct ManageProfileUseCase<'a> {
    profile_repo: &'a dyn ProfileRepository,
    group_repo: &'a dyn GroupRepository,
    state_writer: &'a dyn StateFileWriter,
}

impl<'a> ManageProfileUseCase<'a> {
    pub fn new(
        profile_repo: &'a dyn ProfileRepository,
        group_repo: &'a dyn GroupRepository,
        state_writer: &'a dyn StateFileWriter,
    ) -> Self {
        Self {
            profile_repo,
            group_repo,
            state_writer,
        }
    }

    pub fn create(
        &self,
        name: &str,
        description: &str,
        group_names: Vec<String>,
    ) -> Result<(), DomainError> {
        if self.profile_repo.exists(name)? {
            return Err(DomainError::ProfileAlreadyExists(name.to_string()));
        }
        let mut profile = Profile::new(name, description)?;
        profile.set_groups(group_names);
        self.profile_repo.save(&profile)
    }

    pub fn delete(&self, name: &str) -> Result<(), DomainError> {
        if !self.profile_repo.exists(name)? {
            return Err(DomainError::ProfileNotFound(name.to_string()));
        }
        self.profile_repo.delete(name)
    }

    pub fn list(&self) -> Result<Vec<ProfileInfo>, DomainError> {
        let profiles = self.profile_repo.find_all()?;
        Ok(profiles
            .iter()
            .map(|p| ProfileInfo {
                name: p.name().to_string(),
                description: p.description().to_string(),
                group_names: p.group_names().to_vec(),
            })
            .collect())
    }

    pub fn show(&self, name: &str) -> Result<ProfileInfo, DomainError> {
        let profile = self
            .profile_repo
            .find_by_name(name)?
            .ok_or_else(|| DomainError::ProfileNotFound(name.to_string()))?;
        Ok(ProfileInfo {
            name: profile.name().to_string(),
            description: profile.description().to_string(),
            group_names: profile.group_names().to_vec(),
        })
    }

    /// Activate all groups in the profile (additive mode).
    pub fn activate(&self, name: &str) -> Result<(), DomainError> {
        let profile = self
            .profile_repo
            .find_by_name(name)?
            .ok_or_else(|| DomainError::ProfileNotFound(name.to_string()))?;

        for group_name in profile.group_names() {
            if let Some(mut group) = self.group_repo.find_by_name(group_name)? {
                if !group.is_active() {
                    group.enable();
                    self.group_repo.save(&group)?;
                }
            }
        }

        SyncEnvironmentUseCase::new(self.group_repo, self.state_writer).execute()
    }

    /// Deactivate all groups in the profile.
    pub fn deactivate(&self, name: &str) -> Result<(), DomainError> {
        let profile = self
            .profile_repo
            .find_by_name(name)?
            .ok_or_else(|| DomainError::ProfileNotFound(name.to_string()))?;

        for group_name in profile.group_names() {
            if let Some(mut group) = self.group_repo.find_by_name(group_name)? {
                if group.is_active() {
                    group.disable();
                    self.group_repo.save(&group)?;
                }
            }
        }

        SyncEnvironmentUseCase::new(self.group_repo, self.state_writer).execute()
    }
}
