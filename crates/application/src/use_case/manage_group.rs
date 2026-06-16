use envtools_domain::error::DomainError;
use envtools_domain::model::env_variable::{EnvVariable, PathMode};
use envtools_domain::model::hosts_entry::HostsEntry;
use envtools_domain::model::managed_group::ManagedGroup;
use envtools_domain::repository::GroupRepository;

use crate::dto::{
    AddHostsEntryRequest, AddVariableRequest, CreateGroupRequest, GroupDetail, GroupInfo,
    HostsEntryInfo, VariableInfo,
};

pub struct ManageGroupUseCase<'a> {
    repo: &'a dyn GroupRepository,
}

impl<'a> ManageGroupUseCase<'a> {
    pub fn new(repo: &'a dyn GroupRepository) -> Self {
        Self { repo }
    }

    pub fn create_group(&self, req: CreateGroupRequest) -> Result<(), DomainError> {
        if self.repo.exists(&req.name)? {
            return Err(DomainError::GroupAlreadyExists(req.name));
        }
        let group =
            ManagedGroup::with_kind_and_priority(req.name, req.description, req.kind, req.priority);
        self.repo.save(&group)
    }

    pub fn delete_group(&self, name: &str) -> Result<(), DomainError> {
        if !self.repo.exists(name)? {
            return Err(DomainError::GroupNotFound(name.to_string()));
        }
        self.repo.delete(name)
    }

    pub fn list_groups(&self) -> Result<Vec<GroupInfo>, DomainError> {
        let groups = self.repo.find_all()?;
        Ok(groups
            .iter()
            .map(|g| GroupInfo {
                name: g.name().to_string(),
                description: g.description().to_string(),
                kind: g.kind(),
                active: g.is_active(),
                priority: g.priority().value(),
                variable_count: g.variables().len(),
                hosts_count: g.hosts_entries().len(),
            })
            .collect())
    }

    pub fn show_group(&self, name: &str) -> Result<GroupDetail, DomainError> {
        let group = self
            .repo
            .find_by_name(name)?
            .ok_or_else(|| DomainError::GroupNotFound(name.to_string()))?;

        Ok(GroupDetail {
            name: group.name().to_string(),
            description: group.description().to_string(),
            kind: group.kind(),
            active: group.is_active(),
            priority: group.priority().value(),
            variables: group
                .variables()
                .iter()
                .map(|v| VariableInfo {
                    key: v.key().to_string(),
                    value: v.value().to_string(),
                    path_mode: v.path_mode().clone(),
                })
                .collect(),
            hosts_entries: group
                .hosts_entries()
                .iter()
                .map(|e| HostsEntryInfo {
                    ip: e.ip().to_string(),
                    hostname: e.hostname().to_string(),
                })
                .collect(),
        })
    }

    pub fn add_variable(&self, req: AddVariableRequest) -> Result<(), DomainError> {
        let mut group = self
            .repo
            .find_by_name(&req.group_name)?
            .ok_or_else(|| DomainError::GroupNotFound(req.group_name.clone()))?;

        let var = if req.path_mode == PathMode::Override {
            EnvVariable::new(req.key, req.value)?
        } else {
            EnvVariable::with_path_mode(req.key, req.value, req.path_mode)?
        };

        group.add_variable(var)?;
        self.repo.save(&group)
    }

    pub fn remove_variable(&self, group_name: &str, key: &str) -> Result<(), DomainError> {
        let mut group = self
            .repo
            .find_by_name(group_name)?
            .ok_or_else(|| DomainError::GroupNotFound(group_name.to_string()))?;

        group.remove_variable(key)?;
        self.repo.save(&group)
    }

    pub fn add_hosts_entry(&self, req: AddHostsEntryRequest) -> Result<(), DomainError> {
        let mut group = self
            .repo
            .find_by_name(&req.group_name)?
            .ok_or_else(|| DomainError::GroupNotFound(req.group_name.clone()))?;

        let entry = HostsEntry::new(req.ip, req.hostname)?;
        group.add_hosts_entry(entry)?;
        self.repo.save(&group)
    }

    pub fn remove_hosts_entry(&self, group_name: &str, hostname: &str) -> Result<(), DomainError> {
        let mut group = self
            .repo
            .find_by_name(group_name)?
            .ok_or_else(|| DomainError::GroupNotFound(group_name.to_string()))?;

        group.remove_hosts_entry(hostname)?;
        self.repo.save(&group)
    }
}
