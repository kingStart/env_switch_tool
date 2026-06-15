use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use envtools_domain::error::DomainError;
use envtools_domain::model::env_group::EnvGroup;
use envtools_domain::model::env_variable::{EnvVariable, PathMode};
use envtools_domain::model::priority::Priority;
use envtools_domain::repository::GroupRepository;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    groups: Vec<GroupEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GroupEntry {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    priority: u32,
    #[serde(default)]
    variables: Vec<VariableEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VariableEntry {
    key: String,
    value: String,
    #[serde(default = "default_path_mode")]
    path_mode: String,
}

fn default_path_mode() -> String {
    "override".to_string()
}

impl GroupEntry {
    fn to_domain(&self) -> EnvGroup {
        let variables: Vec<EnvVariable> = self
            .variables
            .iter()
            .filter_map(|v| {
                let mode = match v.path_mode.as_str() {
                    "prepend" => PathMode::Prepend,
                    "append" => PathMode::Append,
                    _ => PathMode::Override,
                };
                EnvVariable::with_path_mode(&v.key, &v.value, mode).ok()
            })
            .collect();

        EnvGroup::from_state(
            self.name.clone(),
            self.description.clone(),
            variables,
            self.active,
            Priority::new(self.priority),
        )
    }

    fn from_domain(group: &EnvGroup) -> Self {
        Self {
            name: group.name().to_string(),
            description: group.description().to_string(),
            active: group.is_active(),
            priority: group.priority().value(),
            variables: group
                .variables()
                .iter()
                .map(|v| VariableEntry {
                    key: v.key().to_string(),
                    value: v.value().to_string(),
                    path_mode: match v.path_mode() {
                        PathMode::Override => "override".to_string(),
                        PathMode::Prepend => "prepend".to_string(),
                        PathMode::Append => "append".to_string(),
                    },
                })
                .collect(),
        }
    }
}

pub struct TomlGroupRepository {
    config_path: PathBuf,
}

impl TomlGroupRepository {
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    pub fn ensure_config_dir(&self) -> Result<(), DomainError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DomainError::GroupNotFound(format!("failed to create config dir: {e}"))
            })?;
        }
        Ok(())
    }

    fn load(&self) -> Result<ConfigFile, DomainError> {
        if !self.config_path.exists() {
            return Ok(ConfigFile { groups: Vec::new() });
        }
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to read config: {e}")))?;
        toml::from_str(&content)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to parse config: {e}")))
    }

    fn persist(&self, config: &ConfigFile) -> Result<(), DomainError> {
        self.ensure_config_dir()?;
        let content = toml::to_string_pretty(config)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to serialize config: {e}")))?;
        fs::write(&self.config_path, content)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to write config: {e}")))
    }
}

impl GroupRepository for TomlGroupRepository {
    fn find_by_name(&self, name: &str) -> Result<Option<EnvGroup>, DomainError> {
        let config = self.load()?;
        Ok(config
            .groups
            .iter()
            .find(|g| g.name == name)
            .map(|g| g.to_domain()))
    }

    fn find_all(&self) -> Result<Vec<EnvGroup>, DomainError> {
        let config = self.load()?;
        Ok(config.groups.iter().map(|g| g.to_domain()).collect())
    }

    fn find_active(&self) -> Result<Vec<EnvGroup>, DomainError> {
        let config = self.load()?;
        Ok(config
            .groups
            .iter()
            .filter(|g| g.active)
            .map(|g| g.to_domain())
            .collect())
    }

    fn save(&self, group: &EnvGroup) -> Result<(), DomainError> {
        let mut config = self.load()?;
        let entry = GroupEntry::from_domain(group);

        if let Some(pos) = config.groups.iter().position(|g| g.name == group.name()) {
            config.groups[pos] = entry;
        } else {
            config.groups.push(entry);
        }

        self.persist(&config)
    }

    fn delete(&self, name: &str) -> Result<(), DomainError> {
        let mut config = self.load()?;
        config.groups.retain(|g| g.name != name);
        self.persist(&config)
    }

    fn exists(&self, name: &str) -> Result<bool, DomainError> {
        let config = self.load()?;
        Ok(config.groups.iter().any(|g| g.name == name))
    }
}
