use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn envtools_cmd(config_dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("envtools").unwrap();
    cmd.arg("--config-dir").arg(config_dir);
    cmd
}

#[test]
fn init_creates_config_dir() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    envtools_cmd(&config_dir)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized envtools"));
}

#[test]
fn group_list_empty() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    envtools_cmd(&config_dir).arg("init").assert().success();

    envtools_cmd(&config_dir)
        .args(["group", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No groups defined"));
}

#[test]
fn full_workflow_create_set_enable_status() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    // Init
    envtools_cmd(&config_dir).arg("init").assert().success();

    // Create group
    envtools_cmd(&config_dir)
        .args(["group", "create", "java", "-d", "Java 17", "-p", "10"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created group: java"));

    // Set variables
    envtools_cmd(&config_dir)
        .args([
            "set",
            "java",
            "JAVA_HOME=/usr/lib/jvm/java-17",
            "+PATH=/usr/lib/jvm/java-17/bin",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated 2 variable(s)"));

    // Show group
    envtools_cmd(&config_dir)
        .args(["group", "show", "java"])
        .assert()
        .success()
        .stdout(predicate::str::contains("JAVA_HOME = /usr/lib/jvm/java-17"))
        .stdout(predicate::str::contains("[prepend]"));

    // Enable
    envtools_cmd(&config_dir)
        .args(["enable", "java"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Enabled: java"));

    // Status
    envtools_cmd(&config_dir)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active groups: java"))
        .stdout(predicate::str::contains("JAVA_HOME = /usr/lib/jvm/java-17"));

    // Group list shows active
    envtools_cmd(&config_dir)
        .args(["group", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[ON]"));

    // Disable
    envtools_cmd(&config_dir)
        .args(["disable", "java"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Disabled: java"));

    // Status shows empty
    envtools_cmd(&config_dir)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No active groups"));
}

#[test]
fn enable_nonexistent_group_fails() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");
    envtools_cmd(&config_dir).arg("init").assert().success();

    envtools_cmd(&config_dir)
        .args(["enable", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn shell_init_bash_outputs_hook() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    envtools_cmd(&config_dir)
        .args(["shell", "init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("__envtools_hook"))
        .stdout(predicate::str::contains("PROMPT_COMMAND"));
}

#[test]
fn shell_init_pwsh_outputs_hook() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    envtools_cmd(&config_dir)
        .args(["shell", "init", "pwsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("__envtools_hook"))
        .stdout(predicate::str::contains("active.ps1"));
}

#[test]
fn unset_variable_from_group() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    envtools_cmd(&config_dir).arg("init").assert().success();
    envtools_cmd(&config_dir)
        .args(["group", "create", "test"])
        .assert()
        .success();
    envtools_cmd(&config_dir)
        .args(["set", "test", "KEY=value"])
        .assert()
        .success();
    envtools_cmd(&config_dir)
        .args(["unset", "test", "KEY"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 1 variable(s)"));

    envtools_cmd(&config_dir)
        .args(["group", "show", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(none)"));
}

#[test]
fn delete_group() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    envtools_cmd(&config_dir).arg("init").assert().success();
    envtools_cmd(&config_dir)
        .args(["group", "create", "temp"])
        .assert()
        .success();
    envtools_cmd(&config_dir)
        .args(["group", "delete", "temp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted group: temp"));
}

#[test]
fn export_to_file_and_import() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");
    let export_file = dir.path().join("export.json");

    // Setup
    envtools_cmd(&config_dir).arg("init").assert().success();
    envtools_cmd(&config_dir)
        .args(["group", "create", "java", "-d", "Java", "-p", "10"])
        .assert()
        .success();
    envtools_cmd(&config_dir)
        .args(["set", "java", "JAVA_HOME=/usr/lib/jvm"])
        .assert()
        .success();

    // Export to file
    envtools_cmd(&config_dir)
        .args(["export", "-o"])
        .arg(&export_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported 1 group(s)"));

    // Verify export file content
    let content = fs::read_to_string(&export_file).unwrap();
    assert!(content.contains("\"name\": \"java\""));
    assert!(content.contains("JAVA_HOME"));

    // Import to a new config
    let config_dir2 = dir.path().join(".envtools2");
    envtools_cmd(&config_dir2).arg("init").assert().success();
    envtools_cmd(&config_dir2)
        .arg("import")
        .arg(&export_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 imported"));

    // Verify imported
    envtools_cmd(&config_dir2)
        .args(["group", "show", "java"])
        .assert()
        .success()
        .stdout(predicate::str::contains("JAVA_HOME = /usr/lib/jvm"));
}

#[test]
fn export_to_stdout() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");

    envtools_cmd(&config_dir).arg("init").assert().success();
    envtools_cmd(&config_dir)
        .args(["group", "create", "test"])
        .assert()
        .success();

    envtools_cmd(&config_dir)
        .args(["export"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"version\": 1"))
        .stdout(predicate::str::contains("\"name\": \"test\""));
}

#[test]
fn import_with_overwrite() {
    let dir = TempDir::new().unwrap();
    let config_dir = dir.path().join(".envtools");
    let export_file = dir.path().join("data.json");

    envtools_cmd(&config_dir).arg("init").assert().success();
    envtools_cmd(&config_dir)
        .args(["group", "create", "dup", "-d", "original"])
        .assert()
        .success();

    // Write export data with same group name but different description
    let json = r#"{"version":1,"groups":[{"name":"dup","description":"updated","active":false,"priority":0,"variables":[]}]}"#;
    fs::write(&export_file, json).unwrap();

    // Import without overwrite - should skip
    envtools_cmd(&config_dir)
        .arg("import")
        .arg(&export_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 skipped"));

    // Import with --overwrite
    envtools_cmd(&config_dir)
        .arg("import")
        .arg(&export_file)
        .arg("--overwrite")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 overwritten"));

    // Verify overwritten
    envtools_cmd(&config_dir)
        .args(["group", "show", "dup"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated"));
}
