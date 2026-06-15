use tempfile::TempDir;

use envtools_domain::model::env_group::EnvGroup;
use envtools_domain::model::env_variable::{EnvVariable, PathMode};
use envtools_domain::repository::GroupRepository;
use envtools_infrastructure::TomlGroupRepository;

fn setup() -> (TempDir, TomlGroupRepository) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    let repo = TomlGroupRepository::new(path);
    (dir, repo)
}

#[test]
fn empty_repo_returns_no_groups() {
    let (_dir, repo) = setup();
    let groups = repo.find_all().unwrap();
    assert!(groups.is_empty());
}

#[test]
fn save_and_find_by_name() {
    let (_dir, repo) = setup();
    let mut group = EnvGroup::new("java", "Java 17");
    group
        .add_variable(EnvVariable::new("JAVA_HOME", "/usr/lib/jvm").unwrap())
        .unwrap();

    repo.save(&group).unwrap();

    let loaded = repo.find_by_name("java").unwrap().unwrap();
    assert_eq!(loaded.name(), "java");
    assert_eq!(loaded.description(), "Java 17");
    assert_eq!(loaded.variables().len(), 1);
    assert_eq!(loaded.variables()[0].key(), "JAVA_HOME");
}

#[test]
fn save_preserves_path_mode() {
    let (_dir, repo) = setup();
    let mut group = EnvGroup::new("go", "");
    group
        .add_variable(
            EnvVariable::with_path_mode("PATH", "/go/bin", PathMode::Prepend).unwrap(),
        )
        .unwrap();

    repo.save(&group).unwrap();

    let loaded = repo.find_by_name("go").unwrap().unwrap();
    assert_eq!(loaded.variables()[0].path_mode(), &PathMode::Prepend);
}

#[test]
fn save_preserves_active_state() {
    let (_dir, repo) = setup();
    let mut group = EnvGroup::new("test", "");
    group.enable();

    repo.save(&group).unwrap();

    let loaded = repo.find_by_name("test").unwrap().unwrap();
    assert!(loaded.is_active());
}

#[test]
fn find_active_filters_correctly() {
    let (_dir, repo) = setup();

    let mut active = EnvGroup::new("active", "");
    active.enable();
    let inactive = EnvGroup::new("inactive", "");

    repo.save(&active).unwrap();
    repo.save(&inactive).unwrap();

    let result = repo.find_active().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name(), "active");
}

#[test]
fn delete_removes_group() {
    let (_dir, repo) = setup();
    let group = EnvGroup::new("temp", "");
    repo.save(&group).unwrap();

    repo.delete("temp").unwrap();

    assert!(!repo.exists("temp").unwrap());
}

#[test]
fn save_updates_existing_group() {
    let (_dir, repo) = setup();
    let group = EnvGroup::new("java", "old desc");
    repo.save(&group).unwrap();

    let mut updated = EnvGroup::new("java", "new desc");
    updated
        .add_variable(EnvVariable::new("KEY", "val").unwrap())
        .unwrap();
    repo.save(&updated).unwrap();

    let loaded = repo.find_by_name("java").unwrap().unwrap();
    assert_eq!(loaded.description(), "new desc");
    assert_eq!(loaded.variables().len(), 1);
}
