use envtools_domain::model::env_variable::{EnvVariable, PathMode};
use envtools_domain::model::managed_group::ManagedGroup;
use envtools_domain::service::group_policy::{GroupPolicy, ResolvedEnvironment};

type EnvGroup = ManagedGroup;

#[test]
fn resolve_single_group() {
    let mut group = EnvGroup::new("java", "");
    group
        .add_variable(EnvVariable::new("JAVA_HOME", "/usr/lib/jvm/java-17").unwrap())
        .unwrap();

    let resolved = GroupPolicy::resolve(&[&group], ":");
    assert_eq!(
        resolved.variables.get("JAVA_HOME").unwrap(),
        "/usr/lib/jvm/java-17"
    );
    assert!(resolved.managed_keys.contains(&"JAVA_HOME".to_string()));
}

#[test]
fn higher_priority_overrides_lower() {
    let mut low = EnvGroup::with_priority("dev", "", 5);
    low.add_variable(EnvVariable::new("ENV", "development").unwrap())
        .unwrap();

    let mut high = EnvGroup::with_priority("prod", "", 10);
    high.add_variable(EnvVariable::new("ENV", "production").unwrap())
        .unwrap();

    let resolved = GroupPolicy::resolve(&[&low, &high], ":");
    assert_eq!(resolved.variables.get("ENV").unwrap(), "production");
}

#[test]
fn prepend_mode_merges_path() {
    let mut group = EnvGroup::new("java", "");
    group
        .add_variable(
            EnvVariable::with_path_mode("PATH", "/usr/lib/jvm/bin", PathMode::Prepend).unwrap(),
        )
        .unwrap();

    let resolved = GroupPolicy::resolve(&[&group], ":");
    let path_val = resolved.variables.get("PATH").unwrap();
    assert!(path_val.starts_with("/usr/lib/jvm/bin:"));
    assert!(path_val.contains("${PATH}"));
}

#[test]
fn append_mode_merges_path() {
    let mut group = EnvGroup::new("go", "");
    group
        .add_variable(
            EnvVariable::with_path_mode("PATH", "/usr/local/go/bin", PathMode::Append).unwrap(),
        )
        .unwrap();

    let resolved = GroupPolicy::resolve(&[&group], ":");
    let path_val = resolved.variables.get("PATH").unwrap();
    assert!(path_val.ends_with("/usr/local/go/bin"));
    assert!(path_val.contains("${PATH}"));
}

#[test]
fn multiple_prepends_from_different_groups() {
    let mut g1 = EnvGroup::with_priority("java", "", 5);
    g1.add_variable(EnvVariable::with_path_mode("PATH", "/java/bin", PathMode::Prepend).unwrap())
        .unwrap();

    let mut g2 = EnvGroup::with_priority("go", "", 10);
    g2.add_variable(EnvVariable::with_path_mode("PATH", "/go/bin", PathMode::Prepend).unwrap())
        .unwrap();

    let resolved = GroupPolicy::resolve(&[&g1, &g2], ":");
    let path_val = resolved.variables.get("PATH").unwrap();
    // Both should be prepended
    assert!(path_val.contains("/java/bin"));
    assert!(path_val.contains("/go/bin"));
}

#[test]
fn windows_separator() {
    let mut group = EnvGroup::new("java", "");
    group
        .add_variable(
            EnvVariable::with_path_mode("PATH", "C:\\Java\\bin", PathMode::Prepend).unwrap(),
        )
        .unwrap();

    let resolved = GroupPolicy::resolve(&[&group], ";");
    let path_val = resolved.variables.get("PATH").unwrap();
    assert!(path_val.contains("C:\\Java\\bin;"));
}

#[test]
fn diff_unset_keys_finds_removed_keys() {
    let mut old_vars = std::collections::HashMap::new();
    old_vars.insert("OLD_KEY".to_string(), "val".to_string());
    old_vars.insert("SHARED".to_string(), "val".to_string());
    let old = ResolvedEnvironment {
        managed_keys: vec!["OLD_KEY".to_string(), "SHARED".to_string()],
        variables: old_vars,
    };

    let mut new_vars = std::collections::HashMap::new();
    new_vars.insert("SHARED".to_string(), "val".to_string());
    new_vars.insert("NEW_KEY".to_string(), "val".to_string());
    let new = ResolvedEnvironment {
        managed_keys: vec!["SHARED".to_string(), "NEW_KEY".to_string()],
        variables: new_vars,
    };

    let unset = GroupPolicy::diff_unset_keys(&old, &new);
    assert_eq!(unset, vec!["OLD_KEY".to_string()]);
}

#[test]
fn empty_groups_produce_empty_result() {
    let resolved = GroupPolicy::resolve(&[], ":");
    assert!(resolved.variables.is_empty());
    assert!(resolved.managed_keys.is_empty());
}
