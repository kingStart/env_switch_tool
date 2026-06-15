use std::fs;
use std::path::PathBuf;

use envtools_domain::error::DomainError;
use envtools_domain::repository::SystemEnvRepository;

/// Linux system environment backend.
/// Writes to ~/.profile or /etc/profile.d/envtools.sh for persistence.
pub struct LinuxEnvRepository {
    profile_path: PathBuf,
}

impl LinuxEnvRepository {
    pub fn user_level() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        Self {
            profile_path: PathBuf::from(home).join(".profile.d").join("envtools.sh"),
        }
    }

    pub fn system_level() -> Self {
        Self {
            profile_path: PathBuf::from("/etc/profile.d/envtools.sh"),
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
        if let Some(parent) = self.profile_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DomainError::GroupNotFound(format!("failed to create profile dir: {e}"))
            })?;
        }

        let mut content = String::from("#!/bin/sh\n# Managed by envtools - DO NOT EDIT\n");
        for (key, value) in vars {
            content.push_str(&format!("export {key}=\"{value}\"\n"));
        }
        fs::write(&self.profile_path, content)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to write profile: {e}")))
    }
}

impl SystemEnvRepository for LinuxEnvRepository {
    fn get(&self, key: &str) -> Result<Option<String>, DomainError> {
        let vars = self.read_profile()?;
        Ok(vars.into_iter().find(|(k, _)| k == key).map(|(_, v)| v))
    }

    fn set(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let mut vars = self.read_profile()?;
        if let Some(pos) = vars.iter().position(|(k, _)| k == key) {
            vars[pos].1 = value.to_string();
        } else {
            vars.push((key.to_string(), value.to_string()));
        }
        self.write_profile(&vars)
    }

    fn remove(&self, key: &str) -> Result<(), DomainError> {
        let mut vars = self.read_profile()?;
        vars.retain(|(k, _)| k != key);
        self.write_profile(&vars)
    }

    fn broadcast_change(&self) -> Result<(), DomainError> {
        // No native broadcast on Linux; shell hooks handle real-time updates
        Ok(())
    }
}
