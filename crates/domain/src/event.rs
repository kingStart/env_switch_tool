use crate::model::env_variable::EnvVariable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    GroupCreated {
        name: String,
    },
    GroupDeleted {
        name: String,
    },
    GroupEnabled {
        name: String,
    },
    GroupDisabled {
        name: String,
    },
    VariableAdded {
        group_name: String,
        variable: EnvVariable,
    },
    VariableRemoved {
        group_name: String,
        key: String,
    },
    VariableUpdated {
        group_name: String,
        variable: EnvVariable,
    },
}
