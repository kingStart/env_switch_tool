use envtools_domain::model::env_variable::PathMode;
use envtools_domain::model::group_kind::GroupKind;

#[derive(Debug, Clone)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: String,
    pub kind: GroupKind,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub struct AddVariableRequest {
    pub group_name: String,
    pub key: String,
    pub value: String,
    pub path_mode: PathMode,
}

#[derive(Debug, Clone)]
pub struct AddHostsEntryRequest {
    pub group_name: String,
    pub ip: String,
    pub hostname: String,
}

#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
    pub kind: GroupKind,
    pub active: bool,
    pub priority: u32,
    pub variable_count: usize,
    pub hosts_count: usize,
}

#[derive(Debug, Clone)]
pub struct GroupDetail {
    pub name: String,
    pub description: String,
    pub kind: GroupKind,
    pub active: bool,
    pub priority: u32,
    pub variables: Vec<VariableInfo>,
    pub hosts_entries: Vec<HostsEntryInfo>,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub key: String,
    pub value: String,
    pub path_mode: PathMode,
}

#[derive(Debug, Clone)]
pub struct HostsEntryInfo {
    pub ip: String,
    pub hostname: String,
}

#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub name: String,
    pub description: String,
    pub group_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EnvironmentSnapshot {
    pub variables: Vec<(String, String)>,
    pub managed_keys: Vec<String>,
}
