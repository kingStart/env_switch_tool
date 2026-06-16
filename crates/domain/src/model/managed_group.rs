use crate::error::DomainError;
use crate::event::DomainEvent;
use crate::model::env_variable::EnvVariable;
use crate::model::group_kind::GroupKind;
use crate::model::hosts_entry::HostsEntry;
use crate::model::priority::Priority;

/// Aggregate Root: a named group that can contain environment variables or hosts entries.
#[derive(Debug, Clone)]
pub struct ManagedGroup {
    name: String,
    description: String,
    kind: GroupKind,
    variables: Vec<EnvVariable>,
    hosts_entries: Vec<HostsEntry>,
    active: bool,
    priority: Priority,
    events: Vec<DomainEvent>,
}

impl ManagedGroup {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        let events = vec![DomainEvent::GroupCreated { name: name.clone() }];
        Self {
            name,
            description: description.into(),
            kind: GroupKind::Env,
            variables: Vec::new(),
            hosts_entries: Vec::new(),
            active: false,
            priority: Priority::default(),
            events,
        }
    }

    pub fn new_hosts(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        let events = vec![DomainEvent::GroupCreated { name: name.clone() }];
        Self {
            name,
            description: description.into(),
            kind: GroupKind::Hosts,
            variables: Vec::new(),
            hosts_entries: Vec::new(),
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

    pub fn with_kind_and_priority(
        name: impl Into<String>,
        description: impl Into<String>,
        kind: GroupKind,
        priority: u32,
    ) -> Self {
        let name = name.into();
        let events = vec![DomainEvent::GroupCreated { name: name.clone() }];
        Self {
            name,
            description: description.into(),
            kind,
            variables: Vec::new(),
            hosts_entries: Vec::new(),
            active: false,
            priority: Priority::new(priority),
            events,
        }
    }

    /// Reconstruct from persistence (no events emitted).
    pub fn from_state(
        name: String,
        description: String,
        kind: GroupKind,
        variables: Vec<EnvVariable>,
        hosts_entries: Vec<HostsEntry>,
        active: bool,
        priority: Priority,
    ) -> Self {
        Self {
            name,
            description,
            kind,
            variables,
            hosts_entries,
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

    pub fn kind(&self) -> GroupKind {
        self.kind
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

    pub fn hosts_entries(&self) -> &[HostsEntry] {
        &self.hosts_entries
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

    // --- Env variable operations ---

    pub fn add_variable(&mut self, variable: EnvVariable) -> Result<Vec<DomainEvent>, DomainError> {
        if self.kind != GroupKind::Env {
            return Err(DomainError::InvalidVariableKey(
                "cannot add variables to a hosts group".to_string(),
            ));
        }
        if self.variables.iter().any(|v| v.key() == variable.key()) {
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

    // --- Hosts entry operations ---

    pub fn add_hosts_entry(&mut self, entry: HostsEntry) -> Result<(), DomainError> {
        if self.kind != GroupKind::Hosts {
            return Err(DomainError::InvalidHostsEntry(
                "cannot add hosts entries to an env group".to_string(),
            ));
        }
        if let Some(pos) = self
            .hosts_entries
            .iter()
            .position(|e| e.hostname() == entry.hostname())
        {
            self.hosts_entries[pos] = entry;
        } else {
            self.hosts_entries.push(entry);
        }
        Ok(())
    }

    pub fn remove_hosts_entry(&mut self, hostname: &str) -> Result<(), DomainError> {
        self.hosts_entries.retain(|e| e.hostname() != hostname);
        Ok(())
    }

    /// Drain all accumulated domain events.
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}
