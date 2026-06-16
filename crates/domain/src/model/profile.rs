use crate::error::DomainError;

#[derive(Debug, Clone)]
pub struct Profile {
    name: String,
    description: String,
    group_names: Vec<String>,
}

impl Profile {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if name.is_empty() {
            return Err(DomainError::InvalidVariableKey(
                "profile name cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            name,
            description: description.into(),
            group_names: Vec::new(),
        })
    }

    pub fn from_state(name: String, description: String, group_names: Vec<String>) -> Self {
        Self {
            name,
            description,
            group_names,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn group_names(&self) -> &[String] {
        &self.group_names
    }

    pub fn add_group(&mut self, group_name: String) {
        if !self.group_names.contains(&group_name) {
            self.group_names.push(group_name);
        }
    }

    pub fn remove_group(&mut self, group_name: &str) {
        self.group_names.retain(|n| n != group_name);
    }

    pub fn set_groups(&mut self, groups: Vec<String>) {
        self.group_names = groups;
    }
}
