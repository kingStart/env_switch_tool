use serde::Serialize;

use envtools_application::dto::{AddVariableRequest, CreateGroupRequest};
use envtools_application::use_case::disable_group::DisableGroupUseCase;
use envtools_application::use_case::enable_group::EnableGroupUseCase;
use envtools_application::use_case::export_import::{ExportData, ExportImportUseCase};
use envtools_application::use_case::manage_group::ManageGroupUseCase;
use envtools_domain::model::env_variable::PathMode;
use envtools_domain::repository::GroupRepository;
use envtools_domain::service::group_policy::GroupPolicy;
use envtools_infrastructure::{FileStateWriter, TomlGroupRepository};

use crate::config_dir;

fn repo() -> TomlGroupRepository {
    let config_path = config_dir().join("config.toml");
    TomlGroupRepository::new(config_path)
}

fn writer() -> FileStateWriter {
    FileStateWriter::new(config_dir())
}

#[derive(Serialize)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
    pub active: bool,
    pub priority: u32,
    pub variable_count: usize,
}

#[derive(Serialize)]
pub struct GroupDetail {
    pub name: String,
    pub description: String,
    pub active: bool,
    pub priority: u32,
    pub variables: Vec<VariableInfo>,
}

#[derive(Serialize)]
pub struct VariableInfo {
    pub key: String,
    pub value: String,
    pub path_mode: String,
}

#[derive(Serialize)]
pub struct StatusInfo {
    pub active_groups: Vec<String>,
    pub variables: Vec<(String, String)>,
}

#[tauri::command]
pub fn get_groups() -> Result<Vec<GroupInfo>, String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    let groups = uc.list_groups().map_err(|e| e.to_string())?;
    Ok(groups
        .into_iter()
        .map(|g| GroupInfo {
            name: g.name,
            description: g.description,
            active: g.active,
            priority: g.priority,
            variable_count: g.variable_count,
        })
        .collect())
}

#[tauri::command]
pub fn get_group_detail(name: String) -> Result<GroupDetail, String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    let detail = uc.show_group(&name).map_err(|e| e.to_string())?;
    Ok(GroupDetail {
        name: detail.name,
        description: detail.description,
        active: detail.active,
        priority: detail.priority,
        variables: detail
            .variables
            .into_iter()
            .map(|v| VariableInfo {
                key: v.key,
                value: v.value,
                path_mode: match v.path_mode {
                    PathMode::Override => "override".to_string(),
                    PathMode::Prepend => "prepend".to_string(),
                    PathMode::Append => "append".to_string(),
                },
            })
            .collect(),
    })
}

#[tauri::command]
pub fn create_group(name: String, description: String, priority: u32) -> Result<(), String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    uc.create_group(CreateGroupRequest {
        name,
        description,
        priority,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_group(name: String) -> Result<(), String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    uc.delete_group(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn enable_group(name: String) -> Result<(), String> {
    let r = repo();
    let w = writer();
    let uc = EnableGroupUseCase::new(&r, &w);
    uc.execute(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disable_group(name: String) -> Result<(), String> {
    let r = repo();
    let w = writer();
    let uc = DisableGroupUseCase::new(&r, &w);
    uc.execute(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_variable(
    group_name: String,
    key: String,
    value: String,
    path_mode: String,
) -> Result<(), String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    let mode = match path_mode.as_str() {
        "prepend" => PathMode::Prepend,
        "append" => PathMode::Append,
        _ => PathMode::Override,
    };
    uc.add_variable(AddVariableRequest {
        group_name: group_name.clone(),
        key,
        value,
        path_mode: mode,
    })
    .map_err(|e| e.to_string())?;
    sync_active_env(&r)
}

#[tauri::command]
pub fn remove_variable(group_name: String, key: String) -> Result<(), String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    uc.remove_variable(&group_name, &key)
        .map_err(|e| e.to_string())?;
    sync_active_env(&r)
}

fn sync_active_env(repo: &dyn GroupRepository) -> Result<(), String> {
    let active = repo.find_active().map_err(|e| e.to_string())?;
    if active.is_empty() {
        return Ok(());
    }
    let w = writer();
    use envtools_application::use_case::sync_environment::SyncEnvironmentUseCase;
    SyncEnvironmentUseCase::new(repo, &w)
        .execute()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_status() -> Result<StatusInfo, String> {
    let r = repo();
    let active_groups = r.find_active().map_err(|e| e.to_string())?;

    if active_groups.is_empty() {
        return Ok(StatusInfo {
            active_groups: Vec::new(),
            variables: Vec::new(),
        });
    }

    let refs: Vec<&_> = active_groups.iter().collect();
    let separator = if cfg!(windows) { ";" } else { ":" };
    let resolved = GroupPolicy::resolve(&refs, separator);

    let mut vars: Vec<(String, String)> = resolved.variables.into_iter().collect();
    vars.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(StatusInfo {
        active_groups: active_groups.iter().map(|g| g.name().to_string()).collect(),
        variables: vars,
    })
}

#[tauri::command]
pub fn export_config(groups: Option<Vec<String>>) -> Result<String, String> {
    let r = repo();
    let uc = ExportImportUseCase::new(&r);
    let filter = groups.as_deref();
    let data = uc.export(filter).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_config(json_data: String, overwrite: bool) -> Result<String, String> {
    let data: ExportData =
        serde_json::from_str(&json_data).map_err(|e| format!("invalid format: {e}"))?;
    let r = repo();
    let uc = ExportImportUseCase::new(&r);
    let result = uc.import(&data, overwrite).map_err(|e| e.to_string())?;
    Ok(format!(
        "Imported: {}, Skipped: {}, Overwritten: {}",
        result.imported, result.skipped, result.overwritten
    ))
}
