use std::fs;
use std::path::Path;

use envtools_application::dto::{AddVariableRequest, CreateGroupRequest};
use envtools_application::port::StateFileWriter;
use envtools_application::use_case::disable_group::DisableGroupUseCase;
use envtools_application::use_case::enable_group::EnableGroupUseCase;
use envtools_application::use_case::export_import::{ExportData, ExportImportUseCase};
use envtools_application::use_case::manage_group::ManageGroupUseCase;
use envtools_domain::error::DomainError;
use envtools_domain::model::env_variable::PathMode;
use envtools_domain::repository::GroupRepository;
use envtools_domain::service::group_policy::GroupPolicy;
use envtools_infrastructure::TomlGroupRepository;

pub fn init(config_dir: &Path, repo: &TomlGroupRepository) -> Result<(), DomainError> {
    repo.ensure_config_dir()?;
    println!("Initialized envtools at: {}", config_dir.display());

    if std::env::var("ENVTOOLS_SKIP_INJECT").is_ok() {
        return Ok(());
    }

    let injected = auto_inject_hooks(config_dir);
    if injected.is_empty() {
        println!();
        println!("Could not auto-detect shell profiles. Add manually:");
        println!("  bash/zsh: eval \"$(envtools shell init bash)\"");
        println!("  fish:     envtools shell init fish | source");
        println!("  pwsh:     envtools shell init pwsh | Invoke-Expression");
    } else {
        println!();
        println!("Shell hooks auto-installed to:");
        for path in &injected {
            println!("  {}", path.display());
        }
        println!();
        println!("Restart your shell or run `source <profile>` to activate.");
    }
    Ok(())
}

const HOOK_MARKER: &str = "# >>> envtools hook >>>";
const HOOK_MARKER_END: &str = "# <<< envtools hook <<<";

fn auto_inject_hooks(_config_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut injected = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    let bashrc = home.join(".bashrc");
    if bashrc.exists() && inject_hook_to_file(&bashrc, "bash").is_ok() {
        injected.push(bashrc);
    }

    let zshrc = home.join(".zshrc");
    if zshrc.exists() && inject_hook_to_file(&zshrc, "zsh").is_ok() {
        injected.push(zshrc);
    }

    let fish_config = home.join(".config").join("fish").join("config.fish");
    let fish_dir_exists = fish_config.parent().map(|p| p.exists()).unwrap_or(false);
    if (fish_config.exists() || fish_dir_exists)
        && inject_hook_to_file(&fish_config, "fish").is_ok()
    {
        injected.push(fish_config);
    }

    if let Some(ps_profile) = get_powershell_profile() {
        if inject_hook_to_file(&ps_profile, "pwsh").is_ok() {
            injected.push(ps_profile);
        }
    }

    injected
}

fn inject_hook_to_file(profile_path: &Path, shell_type: &str) -> Result<(), std::io::Error> {
    let existing = if profile_path.exists() {
        fs::read_to_string(profile_path)?
    } else {
        String::new()
    };

    // Already injected
    if existing.contains(HOOK_MARKER) {
        return Ok(());
    }

    let hook_line = match shell_type {
        "fish" => "envtools shell init fish | source".to_string(),
        "pwsh" => "envtools shell init pwsh | Invoke-Expression".to_string(),
        _ => format!("eval \"$(envtools shell init {shell_type})\""),
    };

    let block = format!("\n{HOOK_MARKER}\n{hook_line}\n{HOOK_MARKER_END}\n");

    // Ensure parent dir exists
    if let Some(parent) = profile_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut content = existing;
    content.push_str(&block);
    fs::write(profile_path, content)
}

fn get_powershell_profile() -> Option<std::path::PathBuf> {
    // Windows: Documents\PowerShell\Microsoft.PowerShell_profile.ps1
    // or Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
    if cfg!(windows) {
        if let Some(home) = dirs::home_dir() {
            let ps_core = home
                .join("Documents")
                .join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1");
            if ps_core.parent().map(|p| p.exists()).unwrap_or(false) {
                return Some(ps_core);
            }
            let ps_legacy = home
                .join("Documents")
                .join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1");
            if ps_legacy.parent().map(|p| p.exists()).unwrap_or(false) {
                return Some(ps_legacy);
            }
        }
    } else {
        // Unix: ~/.config/powershell/Microsoft.PowerShell_profile.ps1
        if let Some(home) = dirs::home_dir() {
            let ps_unix = home
                .join(".config")
                .join("powershell")
                .join("Microsoft.PowerShell_profile.ps1");
            if ps_unix.parent().map(|p| p.exists()).unwrap_or(false) {
                return Some(ps_unix);
            }
        }
    }
    None
}

pub fn group_list(repo: &dyn GroupRepository) -> Result<(), DomainError> {
    let uc = ManageGroupUseCase::new(repo);
    let groups = uc.list_groups()?;

    if groups.is_empty() {
        println!("No groups defined. Create one with: envtools group create <name>");
        return Ok(());
    }

    println!("{:<20} {:<8} {:<8} DESCRIPTION", "NAME", "ACTIVE", "PRIO");
    println!("{}", "-".repeat(60));
    for g in &groups {
        let status = if g.active { "[ON]" } else { "[OFF]" };
        println!(
            "{:<20} {:<8} {:<8} {}",
            g.name, status, g.priority, g.description
        );
    }
    Ok(())
}

pub fn group_create(
    repo: &dyn GroupRepository,
    name: &str,
    description: &str,
    priority: u32,
) -> Result<(), DomainError> {
    let uc = ManageGroupUseCase::new(repo);
    uc.create_group(CreateGroupRequest {
        name: name.to_string(),
        description: description.to_string(),
        priority,
    })?;
    println!("Created group: {name}");
    Ok(())
}

pub fn group_delete(repo: &dyn GroupRepository, name: &str) -> Result<(), DomainError> {
    let uc = ManageGroupUseCase::new(repo);
    uc.delete_group(name)?;
    println!("Deleted group: {name}");
    Ok(())
}

pub fn group_show(repo: &dyn GroupRepository, name: &str) -> Result<(), DomainError> {
    let uc = ManageGroupUseCase::new(repo);
    let detail = uc.show_group(name)?;

    println!("Group: {}", detail.name);
    println!("Description: {}", detail.description);
    println!("Active: {}", detail.active);
    println!("Priority: {}", detail.priority);
    println!("Variables:");
    if detail.variables.is_empty() {
        println!("  (none)");
    } else {
        for v in &detail.variables {
            let mode_tag = match v.path_mode {
                PathMode::Override => "",
                PathMode::Prepend => " [prepend]",
                PathMode::Append => " [append]",
            };
            println!("  {} = {}{}", v.key, v.value, mode_tag);
        }
    }
    Ok(())
}

pub fn set_vars(
    repo: &dyn GroupRepository,
    group: &str,
    vars: &[String],
) -> Result<(), DomainError> {
    let uc = ManageGroupUseCase::new(repo);
    for var_str in vars {
        let (key, value, mode) = parse_var_assignment(var_str)?;
        uc.add_variable(AddVariableRequest {
            group_name: group.to_string(),
            key,
            value,
            path_mode: mode,
        })?;
    }
    println!("Updated {} variable(s) in group '{group}'", vars.len());
    Ok(())
}

pub fn unset_vars(
    repo: &dyn GroupRepository,
    group: &str,
    keys: &[String],
) -> Result<(), DomainError> {
    let uc = ManageGroupUseCase::new(repo);
    for key in keys {
        uc.remove_variable(group, key)?;
    }
    println!("Removed {} variable(s) from group '{group}'", keys.len());
    Ok(())
}

pub fn enable(
    repo: &dyn GroupRepository,
    writer: &dyn StateFileWriter,
    names: &[String],
) -> Result<(), DomainError> {
    for name in names {
        let uc = EnableGroupUseCase::new(repo, writer);
        uc.execute(name)?;
        println!("Enabled: {name}");
    }
    Ok(())
}

pub fn disable(
    repo: &dyn GroupRepository,
    writer: &dyn StateFileWriter,
    names: &[String],
) -> Result<(), DomainError> {
    for name in names {
        let uc = DisableGroupUseCase::new(repo, writer);
        uc.execute(name)?;
        println!("Disabled: {name}");
    }
    Ok(())
}

pub fn status(repo: &dyn GroupRepository) -> Result<(), DomainError> {
    let active_groups = repo.find_active()?;
    if active_groups.is_empty() {
        println!("No active groups.");
        return Ok(());
    }

    let refs: Vec<&_> = active_groups.iter().collect();
    let separator = if cfg!(windows) { ";" } else { ":" };
    let resolved = GroupPolicy::resolve(&refs, separator);

    println!(
        "Active groups: {}",
        active_groups
            .iter()
            .map(|g| g.name())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    println!(
        "Resolved environment ({} variables):",
        resolved.variables.len()
    );
    let mut sorted: Vec<_> = resolved.variables.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());
    for (key, value) in &sorted {
        println!("  {key} = {value}");
    }
    Ok(())
}

pub fn shell_init(config_dir: &Path, shell_type: &str) -> Result<(), DomainError> {
    let script = match shell_type {
        "bash" | "zsh" => generate_bash_hook(config_dir),
        "fish" => generate_fish_hook(config_dir),
        "pwsh" | "powershell" => generate_powershell_hook(config_dir),
        _ => {
            return Err(DomainError::InvalidVariableKey(format!(
                "unsupported shell: {shell_type}. Use: bash, zsh, fish, pwsh"
            )));
        }
    };
    print!("{script}");
    Ok(())
}

fn generate_bash_hook(config_dir: &Path) -> String {
    let state_file = config_dir.join("active.env");
    let sf = state_file.display();
    format!(
        r#"# envtools shell hook (bash/zsh)
__envtools_state_file="{sf}"
__envtools_last_mtime=0

__envtools_hook() {{
    if [[ ! -f "$__envtools_state_file" ]]; then
        return
    fi
    local mtime
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        mtime=$(stat -c %Y "$__envtools_state_file" 2>/dev/null || echo 0)
    else
        mtime=$(stat -f %m "$__envtools_state_file" 2>/dev/null || echo 0)
    fi
    if [[ "$mtime" != "$__envtools_last_mtime" ]]; then
        # Unset previously managed keys
        if [[ -n "$__ENVTOOLS_MANAGED_KEYS" ]]; then
            for key in $__ENVTOOLS_MANAGED_KEYS; do
                unset "$key"
            done
        fi
        source "$__envtools_state_file"
        __envtools_last_mtime="$mtime"
    fi
}}

if [[ -n "$ZSH_VERSION" ]]; then
    autoload -Uz add-zsh-hook
    add-zsh-hook precmd __envtools_hook
else
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="__envtools_hook"
    else
        PROMPT_COMMAND="__envtools_hook;$PROMPT_COMMAND"
    fi
fi

# Initial load
__envtools_hook
"#
    )
}

fn generate_fish_hook(config_dir: &Path) -> String {
    let state_file = config_dir.join("active.fish");
    let sf = state_file.display();
    format!(
        r#"# envtools shell hook (fish)
set -g __envtools_state_file "{sf}"
set -g __envtools_last_mtime 0

function __envtools_hook --on-event fish_prompt
    if not test -f $__envtools_state_file
        return
    end
    set -l mtime (stat -c %Y $__envtools_state_file 2>/dev/null; or stat -f %m $__envtools_state_file 2>/dev/null; or echo 0)
    if test "$mtime" != "$__envtools_last_mtime"
        # Unset previously managed keys
        if set -q __ENVTOOLS_MANAGED_KEYS
            for key in $__ENVTOOLS_MANAGED_KEYS
                set -e $key
            end
        end
        source $__envtools_state_file
        set -g __envtools_last_mtime $mtime
    end
end

# Initial load
__envtools_hook
"#
    )
}

fn generate_powershell_hook(config_dir: &Path) -> String {
    let state_file = config_dir.join("active.ps1");
    let sf = state_file.display();
    format!(
        r#"# envtools shell hook (PowerShell)
$global:__envtools_stateFile = "{sf}"
$global:__envtools_lastMtime = [datetime]::MinValue

function global:__envtools_hook {{
    if (-not (Test-Path $global:__envtools_stateFile)) {{ return }}
    $mt = (Get-Item $global:__envtools_stateFile).LastWriteTimeUtc
    if ($mt -ne $global:__envtools_lastMtime) {{
        # Unset previously managed keys
        if ($env:__ENVTOOLS_MANAGED_KEYS) {{
            foreach ($key in ($env:__ENVTOOLS_MANAGED_KEYS -split ',')) {{
                Remove-Item "Env:\$key" -ErrorAction SilentlyContinue
            }}
        }}
        . $global:__envtools_stateFile
        $global:__envtools_lastMtime = $mt
    }}
}}

# Inject into prompt
$__envtools_originalPrompt = $function:prompt
function global:prompt {{
    __envtools_hook
    & $global:__envtools_originalPrompt
}}

# Initial load
__envtools_hook
"#
    )
}

/// Parse "KEY=VALUE" with optional "+KEY=VALUE" (prepend) or "KEY+=VALUE" (append) syntax.
fn parse_var_assignment(s: &str) -> Result<(String, String, PathMode), DomainError> {
    if let Some(rest) = s.strip_prefix('+') {
        // +KEY=VALUE -> prepend
        let (key, value) = split_kv(rest)?;
        return Ok((key, value, PathMode::Prepend));
    }
    if let Some(idx) = s.find("+=") {
        // KEY+=VALUE -> append
        let key = s[..idx].to_string();
        let value = s[idx + 2..].to_string();
        if key.is_empty() {
            return Err(DomainError::EmptyVariableKey);
        }
        return Ok((key, value, PathMode::Append));
    }
    let (key, value) = split_kv(s)?;
    Ok((key, value, PathMode::Override))
}

fn split_kv(s: &str) -> Result<(String, String), DomainError> {
    let idx = s
        .find('=')
        .ok_or_else(|| DomainError::InvalidVariableKey(format!("expected KEY=VALUE, got: {s}")))?;
    let key = s[..idx].to_string();
    let value = s[idx + 1..].to_string();
    if key.is_empty() {
        return Err(DomainError::EmptyVariableKey);
    }
    Ok((key, value))
}

pub fn export_config(
    repo: &dyn GroupRepository,
    output: Option<&Path>,
    filter: Option<&[String]>,
) -> Result<(), DomainError> {
    let uc = ExportImportUseCase::new(repo);
    let data = uc.export(filter)?;
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| DomainError::InvalidVariableKey(format!("serialization error: {e}")))?;

    match output {
        Some(path) => {
            fs::write(path, &json).map_err(|e| {
                DomainError::GroupNotFound(format!("failed to write export file: {e}"))
            })?;
            println!(
                "Exported {} group(s) to: {}",
                data.groups.len(),
                path.display()
            );
        }
        None => {
            println!("{json}");
        }
    }
    Ok(())
}

pub fn import_config(
    repo: &dyn GroupRepository,
    file: &Path,
    overwrite: bool,
) -> Result<(), DomainError> {
    let content = fs::read_to_string(file)
        .map_err(|e| DomainError::GroupNotFound(format!("failed to read import file: {e}")))?;
    let data: ExportData = serde_json::from_str(&content)
        .map_err(|e| DomainError::InvalidVariableKey(format!("invalid import file format: {e}")))?;

    let uc = ExportImportUseCase::new(repo);
    let result = uc.import(&data, overwrite)?;

    println!(
        "Import complete: {} imported, {} skipped, {} overwritten",
        result.imported, result.skipped, result.overwritten
    );
    Ok(())
}
