use crate::error::DomainError;
use crate::event::DomainEvent;
use crate::model::env_variable::EnvVariable;
use crate::model::priority::Priority;

/// Aggregate Root: a named group of environment variables that can be enabled/disabled as a unit.
#[derive(Debug, Clone)]
pub struct EnvGroup {
    name: String,
    description: String,
    variables: Vec<EnvVariable>,
    active: bool,
    priority: Priority,
    events: Vec<DomainEvent>,
}

impl EnvGroup {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        let events = vec![DomainEvent::GroupCreated { name: name.clone() }];
        Self {
            name,
            description: description.into(),
            variables: Vec::new(),
            active: false,
            priority: Priority::default(),
            events,
        }
    }

    pub fn with_priority(
        name: impl Into<String>,
        description: impl Into<String>,
        priority: u32,
    ) -> Self {
        let mut group = Self::new(name, description);
        group.priority = Priority::new(priority);
        group
    }

    /// Reconstruct from persistence (no events emitted).
    pub fn from_state(
        name: String,
        description: String,
        variables: Vec<EnvVariable>,
        active: bool,
        priority: Priority,
    ) -> Self {
        Self {
            name,
            description,
            variables,
            active,
            priority,
            events: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn variables(&self) -> &[EnvVariable] {
        &self.variables
    }

    pub fn set_description(&mut self, desc: impl Into<String>) {
        self.description = desc.into();
    }

    pub fn set_priority(&mut self, priority: u32) {
        self.priority = Priority::new(priority);
    }

    pub fn enable(&mut self) -> Vec<DomainEvent> {
        if self.active {
            return Vec::new();
        }
        self.active = true;
        let event = DomainEvent::GroupEnabled {
            name: self.name.clone(),
        };
        self.events.push(event.clone());
        vec![event]
    }

    pub fn disable(&mut self) -> Vec<DomainEvent> {
        if !self.active {
            return Vec::new();
        }
        self.active = false;
        let event = DomainEvent::GroupDisabled {
            name: self.name.clone(),
        };
        self.events.push(event.clone());
        vec![event]
    }

    pub fn add_variable(&mut self, variable: EnvVariable) -> Result<Vec<DomainEvent>, DomainError> {
        if self.variables.iter().any(|v| v.key() == variable.key()) {
            // Update existing
            let idx = self
                .variables
                .iter()
                .position(|v| v.key() == variable.key())
                .unwrap();
            self.variables[idx] = variable.clone();
            let event = DomainEvent::VariableUpdated {
                group_name: self.name.clone(),
                variable,
            };
            self.events.push(event.clone());
            return Ok(vec![event]);
        }
        let event = DomainEvent::VariableAdded {
            group_name: self.name.clone(),
            variable: variable.clone(),
        };
        self.variables.push(variable);
        self.events.push(event.clone());
        Ok(vec![event])
    }

    pub fn remove_variable(&mut self, key: &str) -> Result<Vec<DomainEvent>, DomainError> {
        let pos = self.variables.iter().position(|v| v.key() == key);
        match pos {
            Some(idx) => {
                self.variables.remove(idx);
                let event = DomainEvent::VariableRemoved {
                    group_name: self.name.clone(),
                    key: key.to_string(),
                };
                self.events.push(event.clone());
                Ok(vec![event])
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn get_variable(&self, key: &str) -> Option<&EnvVariable> {
        self.variables.iter().find(|v| v.key() == key)
    }

    /// Drain all accumulated domain events.
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}
