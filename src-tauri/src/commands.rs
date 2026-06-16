use serde::Serialize;

use envtools_application::dto::{AddHostsEntryRequest, AddVariableRequest, CreateGroupRequest};
use envtools_application::use_case::disable_group::DisableGroupUseCase;
use envtools_application::use_case::enable_group::EnableGroupUseCase;
use envtools_application::use_case::export_import::{ExportData, ExportImportUseCase};
use envtools_application::use_case::manage_group::ManageGroupUseCase;
use envtools_application::use_case::manage_profile::ManageProfileUseCase;
use envtools_domain::model::env_variable::PathMode;
use envtools_domain::model::group_kind::GroupKind;
use envtools_domain::repository::{GroupRepository, ProfileRepository};
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
    pub kind: String,
    pub active: bool,
    pub priority: u32,
    pub variable_count: usize,
    pub hosts_count: usize,
}

#[derive(Serialize)]
pub struct GroupDetail {
    pub name: String,
    pub description: String,
    pub kind: String,
    pub active: bool,
    pub priority: u32,
    pub variables: Vec<VariableInfo>,
    pub hosts_entries: Vec<HostsEntryInfo>,
}

#[derive(Serialize)]
pub struct VariableInfo {
    pub key: String,
    pub value: String,
    pub path_mode: String,
}

#[derive(Serialize)]
pub struct HostsEntryInfo {
    pub ip: String,
    pub hostname: String,
}

#[derive(Serialize)]
pub struct ProfileInfo {
    pub name: String,
    pub description: String,
    pub group_names: Vec<String>,
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
            kind: g.kind.to_string(),
            active: g.active,
            priority: g.priority,
            variable_count: g.variable_count,
            hosts_count: g.hosts_count,
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
        kind: detail.kind.to_string(),
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
        hosts_entries: detail
            .hosts_entries
            .into_iter()
            .map(|e| HostsEntryInfo {
                ip: e.ip,
                hostname: e.hostname,
            })
            .collect(),
    })
}

#[tauri::command]
pub fn create_group(
    name: String,
    description: String,
    kind: String,
    priority: u32,
) -> Result<(), String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    uc.create_group(CreateGroupRequest {
        name,
        description,
        kind: GroupKind::parse(&kind),
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

#[tauri::command]
pub fn add_hosts_entry(group_name: String, ip: String, hostname: String) -> Result<(), String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    uc.add_hosts_entry(AddHostsEntryRequest {
        group_name,
        ip,
        hostname,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_hosts_entry(group_name: String, hostname: String) -> Result<(), String> {
    let r = repo();
    let uc = ManageGroupUseCase::new(&r);
    uc.remove_hosts_entry(&group_name, &hostname)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_hosts() -> Result<(), String> {
    use envtools_application::port::HostsFileWriter;
    use envtools_infrastructure::SystemHostsFileWriter;

    let r = repo();
    let active_groups = r.find_active().map_err(|e| e.to_string())?;
    let hosts_entries: Vec<_> = active_groups
        .iter()
        .filter(|g| g.kind() == GroupKind::Hosts)
        .flat_map(|g| g.hosts_entries().iter().cloned())
        .collect();

    let hw = SystemHostsFileWriter::new();
    if hosts_entries.is_empty() {
        hw.clear_managed().map_err(|e| e.to_string())
    } else {
        hw.apply_hosts(&hosts_entries).map_err(|e| e.to_string())
    }
}

// --- Profile commands ---

#[tauri::command]
pub fn get_profiles() -> Result<Vec<ProfileInfo>, String> {
    let r = repo();
    let profiles = ProfileRepository::find_all(&r).map_err(|e| e.to_string())?;
    Ok(profiles
        .into_iter()
        .map(|p| ProfileInfo {
            name: p.name().to_string(),
            description: p.description().to_string(),
            group_names: p.group_names().to_vec(),
        })
        .collect())
}

#[tauri::command]
pub fn create_profile(
    name: String,
    description: String,
    group_names: Vec<String>,
) -> Result<(), String> {
    let r = repo();
    let w = writer();
    let uc = ManageProfileUseCase::new(&r, &r, &w);
    uc.create(&name, &description, group_names)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_profile(name: String) -> Result<(), String> {
    let r = repo();
    let w = writer();
    let uc = ManageProfileUseCase::new(&r, &r, &w);
    uc.delete(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn activate_profile(name: String) -> Result<(), String> {
    let r = repo();
    let w = writer();
    let uc = ManageProfileUseCase::new(&r, &r, &w);
    uc.activate(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn deactivate_profile(name: String) -> Result<(), String> {
    let r = repo();
    let w = writer();
    let uc = ManageProfileUseCase::new(&r, &r, &w);
    uc.deactivate(&name).map_err(|e| e.to_string())
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
