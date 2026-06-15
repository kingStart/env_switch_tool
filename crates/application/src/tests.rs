use envtools_domain::error::DomainError;
use envtools_domain::model::env_group::EnvGroup;
use envtools_domain::model::env_variable::{EnvVariable, PathMode};
use envtools_domain::repository::GroupRepository;
use envtools_domain::service::group_policy::ResolvedEnvironment;

use crate::dto::{AddVariableRequest, CreateGroupRequest};
use crate::port::StateFileWriter;
use crate::use_case::disable_group::DisableGroupUseCase;
use crate::use_case::enable_group::EnableGroupUseCase;
use crate::use_case::manage_group::ManageGroupUseCase;

use std::collections::HashMap;
use std::sync::RwLock;

// --- Mock Repository ---
struct MockRepo {
    groups: RwLock<HashMap<String, EnvGroup>>,
}

impl MockRepo {
    fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
        }
    }

    fn with_group(self, group: EnvGroup) -> Self {
        self.groups
            .write()
            .unwrap()
            .insert(group.name().to_string(), group);
        self
    }
}

impl GroupRepository for MockRepo {
    fn find_by_name(&self, name: &str) -> Result<Option<EnvGroup>, DomainError> {
        Ok(self.groups.read().unwrap().get(name).cloned())
    }

    fn find_all(&self) -> Result<Vec<EnvGroup>, DomainError> {
        Ok(self.groups.read().unwrap().values().cloned().collect())
    }

    fn find_active(&self) -> Result<Vec<EnvGroup>, DomainError> {
        Ok(self
            .groups
            .read()
            .unwrap()
            .values()
            .filter(|g| g.is_active())
            .cloned()
            .collect())
    }

    fn save(&self, group: &EnvGroup) -> Result<(), DomainError> {
        self.groups
            .write()
            .unwrap()
            .insert(group.name().to_string(), group.clone());
        Ok(())
    }

    fn delete(&self, name: &str) -> Result<(), DomainError> {
        self.groups.write().unwrap().remove(name);
        Ok(())
    }

    fn exists(&self, name: &str) -> Result<bool, DomainError> {
        Ok(self.groups.read().unwrap().contains_key(name))
    }
}

// --- Mock StateFileWriter ---
struct MockStateWriter {
    bash_calls: RwLock<Vec<ResolvedEnvironment>>,
    ps_calls: RwLock<Vec<ResolvedEnvironment>>,
    fish_calls: RwLock<Vec<ResolvedEnvironment>>,
}

impl MockStateWriter {
    fn new() -> Self {
        Self {
            bash_calls: RwLock::new(Vec::new()),
            ps_calls: RwLock::new(Vec::new()),
            fish_calls: RwLock::new(Vec::new()),
        }
    }
}

impl StateFileWriter for MockStateWriter {
    fn write_bash(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError> {
        self.bash_calls.write().unwrap().push(resolved.clone());
        Ok(())
    }

    fn write_powershell(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError> {
        self.ps_calls.write().unwrap().push(resolved.clone());
        Ok(())
    }

    fn write_fish(&self, resolved: &ResolvedEnvironment) -> Result<(), DomainError> {
        self.fish_calls.write().unwrap().push(resolved.clone());
        Ok(())
    }
}

// === ManageGroupUseCase Tests ===

#[test]
fn create_group_success() {
    let repo = MockRepo::new();
    let uc = ManageGroupUseCase::new(&repo);

    uc.create_group(CreateGroupRequest {
        name: "java".to_string(),
        description: "Java env".to_string(),
        priority: 10,
    })
    .unwrap();

    assert!(repo.exists("java").unwrap());
}

#[test]
fn create_duplicate_group_fails() {
    let group = EnvGroup::new("java", "");
    let repo = MockRepo::new().with_group(group);
    let uc = ManageGroupUseCase::new(&repo);

    let result = uc.create_group(CreateGroupRequest {
        name: "java".to_string(),
        description: "".to_string(),
        priority: 0,
    });

    assert!(matches!(result, Err(DomainError::GroupAlreadyExists(_))));
}

#[test]
fn delete_nonexistent_group_fails() {
    let repo = MockRepo::new();
    let uc = ManageGroupUseCase::new(&repo);

    let result = uc.delete_group("nope");
    assert!(matches!(result, Err(DomainError::GroupNotFound(_))));
}

#[test]
fn list_groups_returns_all() {
    let g1 = EnvGroup::new("a", "desc a");
    let g2 = EnvGroup::new("b", "desc b");
    let repo = MockRepo::new().with_group(g1).with_group(g2);
    let uc = ManageGroupUseCase::new(&repo);

    let list = uc.list_groups().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn show_group_returns_detail() {
    let mut group = EnvGroup::with_priority("java", "Java 17", 10);
    group
        .add_variable(EnvVariable::new("JAVA_HOME", "/usr/lib/jvm").unwrap())
        .unwrap();
    let repo = MockRepo::new().with_group(group);
    let uc = ManageGroupUseCase::new(&repo);

    let detail = uc.show_group("java").unwrap();
    assert_eq!(detail.name, "java");
    assert_eq!(detail.priority, 10);
    assert_eq!(detail.variables.len(), 1);
    assert_eq!(detail.variables[0].key, "JAVA_HOME");
}

#[test]
fn add_variable_to_group() {
    let group = EnvGroup::new("java", "");
    let repo = MockRepo::new().with_group(group);
    let uc = ManageGroupUseCase::new(&repo);

    uc.add_variable(AddVariableRequest {
        group_name: "java".to_string(),
        key: "JAVA_HOME".to_string(),
        value: "/usr/lib/jvm".to_string(),
        path_mode: PathMode::Override,
    })
    .unwrap();

    let saved = repo.find_by_name("java").unwrap().unwrap();
    assert_eq!(saved.variables().len(), 1);
}

#[test]
fn add_variable_to_nonexistent_group_fails() {
    let repo = MockRepo::new();
    let uc = ManageGroupUseCase::new(&repo);

    let result = uc.add_variable(AddVariableRequest {
        group_name: "nope".to_string(),
        key: "KEY".to_string(),
        value: "val".to_string(),
        path_mode: PathMode::Override,
    });

    assert!(matches!(result, Err(DomainError::GroupNotFound(_))));
}

// === EnableGroupUseCase Tests ===

#[test]
fn enable_group_activates_and_syncs() {
    let group = EnvGroup::new("java", "");
    let repo = MockRepo::new().with_group(group);
    let writer = MockStateWriter::new();

    let uc = EnableGroupUseCase::new(&repo, &writer);
    uc.execute("java").unwrap();

    let saved = repo.find_by_name("java").unwrap().unwrap();
    assert!(saved.is_active());
    assert_eq!(writer.bash_calls.read().unwrap().len(), 1);
    assert_eq!(writer.ps_calls.read().unwrap().len(), 1);
    assert_eq!(writer.fish_calls.read().unwrap().len(), 1);
}

#[test]
fn enable_nonexistent_group_fails() {
    let repo = MockRepo::new();
    let writer = MockStateWriter::new();
    let uc = EnableGroupUseCase::new(&repo, &writer);

    let result = uc.execute("nope");
    assert!(matches!(result, Err(DomainError::GroupNotFound(_))));
}

// === DisableGroupUseCase Tests ===

#[test]
fn disable_group_deactivates_and_syncs() {
    let mut group = EnvGroup::new("java", "");
    group.enable();
    let repo = MockRepo::new().with_group(group);
    let writer = MockStateWriter::new();

    let uc = DisableGroupUseCase::new(&repo, &writer);
    uc.execute("java").unwrap();

    let saved = repo.find_by_name("java").unwrap().unwrap();
    assert!(!saved.is_active());
    assert_eq!(writer.bash_calls.read().unwrap().len(), 1);
}
