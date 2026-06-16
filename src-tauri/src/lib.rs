mod commands;
mod tray;

use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".envtools")
}

pub fn run() {
    ensure_initialized();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_groups,
            commands::get_group_detail,
            commands::create_group,
            commands::delete_group,
            commands::enable_group,
            commands::disable_group,
            commands::set_variable,
            commands::remove_variable,
            commands::add_hosts_entry,
            commands::remove_hosts_entry,
            commands::sync_hosts,
            commands::get_profiles,
            commands::create_profile,
            commands::delete_profile,
            commands::activate_profile,
            commands::deactivate_profile,
            commands::get_status,
            commands::export_config,
            commands::import_config,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn ensure_initialized() {
    let dir = config_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }

    #[cfg(windows)]
    inject_powershell_hook(&dir);
}

#[cfg(windows)]
fn inject_powershell_hook(config_dir: &std::path::Path) {
    use std::process::Command;
    let _ = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Set-ExecutionPolicy RemoteSigned -Scope CurrentUser -Force",
        ])
        .output();

    let home = dirs::home_dir().unwrap_or_default();
    let docs = home.join("Documents");

    let profile_dirs = [docs.join("PowerShell"), docs.join("WindowsPowerShell")];

    for profile_dir in &profile_dirs {
        if std::fs::create_dir_all(profile_dir).is_err() {
            continue;
        }
        let profile = profile_dir.join("Microsoft.PowerShell_profile.ps1");
        let mut content = std::fs::read_to_string(&profile).unwrap_or_default();

        // Remove old-style hook (pipe approach that doesn't work)
        if content.contains("envtools shell init pwsh | Invoke-Expression") {
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut in_old_hook = false;
            for line in &lines {
                if line.contains(">>> envtools hook >>>") {
                    in_old_hook = true;
                    continue;
                }
                if line.contains("<<< envtools hook <<<") {
                    in_old_hook = false;
                    continue;
                }
                if !in_old_hook {
                    new_lines.push(*line);
                }
            }
            content = new_lines.join("\r\n");
        }

        if content.contains("__envtools_hook") {
            continue;
        }
        let state_file = config_dir.join("active.ps1").display().to_string();
        let hook = format!(
            r#"
# >>> envtools hook >>>
$global:__envtools_stateFile = "{}"
$global:__envtools_lastMtime = [datetime]::MinValue
function global:__envtools_hook {{
    if (-not (Test-Path $global:__envtools_stateFile)) {{ return }}
    $mt = (Get-Item $global:__envtools_stateFile).LastWriteTimeUtc
    if ($mt -ne $global:__envtools_lastMtime) {{
        if ($env:__ENVTOOLS_MANAGED_KEYS) {{
            foreach ($key in ($env:__ENVTOOLS_MANAGED_KEYS -split ',')) {{
                Remove-Item "Env:\$key" -ErrorAction SilentlyContinue
            }}
        }}
        . $global:__envtools_stateFile
        $global:__envtools_lastMtime = $mt
    }}
}}
$__envtools_originalPrompt = $function:prompt
function global:prompt {{
    __envtools_hook
    & $global:__envtools_originalPrompt
}}
__envtools_hook
# <<< envtools hook <<<
"#,
            state_file
        );
        let new_content = format!("{content}{hook}");
        let _ = std::fs::write(&profile, new_content);
    }
}
