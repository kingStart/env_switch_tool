use envtools_domain::error::DomainError;
use envtools_domain::repository::SystemEnvRepository;

use winreg::enums::*;
use winreg::RegKey;

pub struct WindowsEnvRepository {
    system_level: bool,
}

impl WindowsEnvRepository {
    /// If `system_level` is true, operates on HKLM (requires admin).
    /// Otherwise operates on HKCU (current user).
    pub fn new(system_level: bool) -> Self {
        Self { system_level }
    }

    fn open_key(&self) -> Result<RegKey, DomainError> {
        let hkey = if self.system_level {
            RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey_with_flags(
                    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                    KEY_READ | KEY_WRITE,
                )
        } else {
            RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        };

        hkey.map_err(|e| {
            DomainError::GroupNotFound(format!("failed to open registry key: {e}"))
        })
    }
}

impl SystemEnvRepository for WindowsEnvRepository {
    fn get(&self, key: &str) -> Result<Option<String>, DomainError> {
        let reg_key = self.open_key()?;
        match reg_key.get_value::<String, _>(key) {
            Ok(val) => Ok(Some(val)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DomainError::GroupNotFound(format!(
                "failed to read registry value '{key}': {e}"
            ))),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let reg_key = self.open_key()?;
        reg_key.set_value(key, &value).map_err(|e| {
            DomainError::GroupNotFound(format!("failed to set registry value '{key}': {e}"))
        })
    }

    fn remove(&self, key: &str) -> Result<(), DomainError> {
        let reg_key = self.open_key()?;
        match reg_key.delete_value(key) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(DomainError::GroupNotFound(format!(
                "failed to delete registry value '{key}': {e}"
            ))),
        }
    }

    fn broadcast_change(&self) -> Result<(), DomainError> {
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::core::*;

        unsafe {
            let mut result: usize = 0;
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(w!("Environment").as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                5000,
                Some(&mut result),
            );
        }
        Ok(())
    }
}
