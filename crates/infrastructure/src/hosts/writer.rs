use std::fs;
use std::path::PathBuf;

use envtools_application::port::HostsFileWriter;
use envtools_domain::error::DomainError;
use envtools_domain::model::hosts_entry::HostsEntry;

const MARKER_START: &str = "# >>> envtools managed >>>";
const MARKER_END: &str = "# <<< envtools managed <<<";

pub struct SystemHostsFileWriter {
    hosts_path: PathBuf,
}

impl Default for SystemHostsFileWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemHostsFileWriter {
    pub fn new() -> Self {
        let hosts_path = if cfg!(windows) {
            PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
        } else {
            PathBuf::from("/etc/hosts")
        };
        Self { hosts_path }
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self { hosts_path: path }
    }

    fn read_hosts(&self) -> Result<String, DomainError> {
        if !self.hosts_path.exists() {
            return Ok(String::new());
        }
        fs::read_to_string(&self.hosts_path)
            .map_err(|e| DomainError::GroupNotFound(format!("failed to read hosts file: {e}")))
    }

    fn write_hosts(&self, content: &str) -> Result<(), DomainError> {
        fs::write(&self.hosts_path, content).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                DomainError::ElevationRequired
            } else {
                DomainError::GroupNotFound(format!("failed to write hosts file: {e}"))
            }
        })
    }

    fn strip_managed_block(content: &str) -> String {
        let mut result = Vec::new();
        let mut in_block = false;

        for line in content.lines() {
            if line.trim() == MARKER_START {
                in_block = true;
                continue;
            }
            if line.trim() == MARKER_END {
                in_block = false;
                continue;
            }
            if !in_block {
                result.push(line);
            }
        }

        let mut out = result.join("\n");
        if !out.ends_with('\n') && !out.is_empty() {
            out.push('\n');
        }
        out
    }

    fn build_managed_block(entries: &[HostsEntry]) -> String {
        if entries.is_empty() {
            return String::new();
        }
        let mut block = format!("{MARKER_START}\n");
        for entry in entries {
            block.push_str(&format!("{} {}\n", entry.ip(), entry.hostname()));
        }
        block.push_str(&format!("{MARKER_END}\n"));
        block
    }
}

impl HostsFileWriter for SystemHostsFileWriter {
    fn apply_hosts(&self, entries: &[HostsEntry]) -> Result<(), DomainError> {
        let content = self.read_hosts()?;
        let clean = Self::strip_managed_block(&content);
        let block = Self::build_managed_block(entries);
        let new_content = format!("{clean}{block}");
        self.write_hosts(&new_content)
    }

    fn clear_managed(&self) -> Result<(), DomainError> {
        let content = self.read_hosts()?;
        let clean = Self::strip_managed_block(&content);
        self.write_hosts(&clean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_managed_block() {
        let content = "127.0.0.1 localhost\n# >>> envtools managed >>>\n192.168.1.1 test.local\n# <<< envtools managed <<<\n::1 localhost\n";
        let result = SystemHostsFileWriter::strip_managed_block(content);
        assert_eq!(result, "127.0.0.1 localhost\n::1 localhost\n");
    }

    #[test]
    fn test_build_managed_block() {
        let entries = vec![
            HostsEntry::new("127.0.0.1", "api.local").unwrap(),
            HostsEntry::new("10.0.0.1", "db.local").unwrap(),
        ];
        let block = SystemHostsFileWriter::build_managed_block(&entries);
        assert!(block.contains("# >>> envtools managed >>>"));
        assert!(block.contains("127.0.0.1 api.local"));
        assert!(block.contains("10.0.0.1 db.local"));
        assert!(block.contains("# <<< envtools managed <<<"));
    }
}
