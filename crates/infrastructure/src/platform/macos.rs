use std::fs;
use std::path::PathBuf;
use std::process::Command;

use envtools_domain::error::DomainError;
use envtools_domain::repository::SystemEnvRepository;

/// macOS system environment backend.
/// Uses `launchctl setenv` for GUI apps and ~/.zprofile for shell persistence.
pub struct MacOsEnvRepository {
    profile_path: PathBuf,
}

impl MacOsEnvRepository {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/Shared".to_string());
        Self {
            profile_path: PathBuf::from(home).join(".envtools_profile"),
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { profile_path: path }
    }

    fn read_profile(&self) -> Result<Vec<(String, String)>, DomainError> {
        if !self.profile_path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.profile_path)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to read profile: {e}")))?;

        let mut vars = Vec::new();
        for line in content.lines() {
            if let Some(assignment) = line.strip_prefix("export ") {
                if let Some(eq_pos) = assignment.find('=') {
                    let key = assignment[..eq_pos].to_string();
                    let value = assignment[eq_pos + 1..].trim_matches('"').to_string();
                    vars.push((key, value));
                }
            }
        }
        Ok(vars)
    }

    fn write_profile(&self, vars: &[(String, String)]) -> Result<(), DomainError> {
        let mut content = String::from("# Managed by envtools - DO NOT EDIT\n");
        for (key, value) in vars {
            content.push_str(&format!("export {key}=\"{value}\"\n"));
        }
        fs::write(&self.profile_path, content)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to write profile: {e}")))
    }
}

impl SystemEnvRepository for MacOsEnvRepository {
    fn get(&self, key: &str) -> Result<Option<String>, DomainError> {
        let vars = self.read_profile()?;
        Ok(vars.into_iter().find(|(k, _)| k == key).map(|(_, v)| v))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), DomainError> {
        // Set via launchctl for GUI apps
        let _ = Command::new("launchctl")
            .args(["setenv", key, value])
            .output();

        let mut vars = self.read_profile()?;
        if let Some(pos) = vars.iter().position(|(k, _)| k == key) {
            vars[pos].1 = value.to_string();
        } else {
            vars.push((key.to_string(), value.to_string()));
        }
        self.write_profile(&vars)
    }

    fn remove(&self, key: &str) -> Result<(), DomainError> {
        let _ = Command::new("launchctl").args(["unsetenv", key]).output();

        let mut vars = self.read_profile()?;
        vars.retain(|(k, _)| k != key);
        self.write_profile(&vars)
    }

    fn broadcast_change(&self) -> Result<(), DomainError> {
        // No native broadcast on macOS; launchctl + shell hooks handle updates
        Ok(())
    }
}
