use envtools_domain::model::env_variable::PathMode;

#[derive(Debug, Clone)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: String,
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
pub struct GroupInfo {
    pub name: String,
    pub description: String,
    pub active: bool,
    pub priority: u32,
    pub variable_count: usize,
}

#[derive(Debug, Clone)]
pub struct GroupDetail {
    pub name: String,
    pub description: String,
    pub active: bool,
    pub priority: u32,
    pub variables: Vec<VariableInfo>,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub key: String,
    pub value: String,
    pub path_mode: PathMode,
}

#[derive(Debug, Clone)]
pub struct EnvironmentSnapshot {
    pub variables: Vec<(String, String)>,
    pub managed_keys: Vec<String>,
}
