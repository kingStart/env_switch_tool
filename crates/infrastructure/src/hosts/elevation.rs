use std::process::Command;

use envtools_application::port::ElevationService;
use envtools_domain::error::DomainError;

#[derive(Default)]
pub struct PlatformElevationService;

impl PlatformElevationService {
    pub fn new() -> Self {
        Self
    }
}

impl ElevationService for PlatformElevationService {
    fn run_elevated(&self, args: &[&str]) -> Result<(), DomainError> {
        let exe = std::env::current_exe()
            .map_err(|e| DomainError::GroupNotFound(format!("cannot find self: {e}")))?;

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let args_str = args.join(" ");
            let status = Command::new("powershell")
                .args([
                    "-Command",
                    &format!(
                        "Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -Wait",
                        exe.display(),
                        args_str
                    ),
                ])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .status()
                .map_err(|e| DomainError::GroupNotFound(format!("elevation failed: {e}")))?;

            if status.success() {
                Ok(())
            } else {
                Err(DomainError::ElevationRequired)
            }
        }

        #[cfg(target_os = "linux")]
        {
            let mut cmd_args = vec![exe.to_str().unwrap_or("envtools")];
            cmd_args.extend_from_slice(args);
            let status = Command::new("pkexec")
                .args(&cmd_args)
                .status()
                .map_err(|e| DomainError::GroupNotFound(format!("pkexec failed: {e}")))?;

            if status.success() {
                Ok(())
            } else {
                Err(DomainError::ElevationRequired)
            }
        }

        #[cfg(target_os = "macos")]
        {
            let args_str = std::iter::once(exe.to_string_lossy().to_string())
                .chain(args.iter().map(|a| a.to_string()))
                .collect::<Vec<_>>()
                .join(" ");

            let script = format!(
                "do shell script \"{}\" with administrator privileges",
                args_str.replace('\\', "\\\\").replace('"', "\\\"")
            );

            let status = Command::new("osascript")
                .args(["-e", &script])
                .status()
                .map_err(|e| DomainError::GroupNotFound(format!("osascript failed: {e}")))?;

            if status.success() {
                Ok(())
            } else {
                Err(DomainError::ElevationRequired)
            }
        }
    }
}
