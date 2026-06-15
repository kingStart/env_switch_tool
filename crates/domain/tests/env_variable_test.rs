use envtools_domain::error::DomainError;
use envtools_domain::model::env_variable::{EnvVariable, PathMode};

#[test]
fn create_valid_variable() {
    let var = EnvVariable::new("JAVA_HOME", "/usr/lib/jvm/java-17").unwrap();
    assert_eq!(var.key(), "JAVA_HOME");
    assert_eq!(var.value(), "/usr/lib/jvm/java-17");
    assert_eq!(var.path_mode(), &PathMode::Override);
}

#[test]
fn create_variable_with_path_mode() {
    let var = EnvVariable::with_path_mode("PATH", "/usr/local/bin", PathMode::Prepend).unwrap();
    assert_eq!(var.key(), "PATH");
    assert_eq!(var.path_mode(), &PathMode::Prepend);
}

#[test]
fn reject_empty_key() {
    let result = EnvVariable::new("", "value");
    assert_eq!(result.unwrap_err(), DomainError::EmptyVariableKey);
}

#[test]
fn reject_key_starting_with_digit() {
    let result = EnvVariable::new("1INVALID", "value");
    assert!(matches!(result, Err(DomainError::InvalidVariableKey(_))));
}

#[test]
fn reject_key_with_special_chars() {
    let result = EnvVariable::new("MY-VAR", "value");
    assert!(matches!(result, Err(DomainError::InvalidVariableKey(_))));
}

#[test]
fn accept_underscore_prefix() {
    let var = EnvVariable::new("_PRIVATE", "val").unwrap();
    assert_eq!(var.key(), "_PRIVATE");
}

#[test]
fn display_format() {
    let var = EnvVariable::new("KEY", "value").unwrap();
    assert_eq!(format!("{var}"), "KEY=value");
}

#[test]
fn set_value_updates_correctly() {
    let mut var = EnvVariable::new("KEY", "old").unwrap();
    var.set_value("new");
    assert_eq!(var.value(), "new");
}
