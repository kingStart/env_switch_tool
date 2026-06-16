use crate::error::DomainError;
use std::fmt;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostsEntry {
    ip: String,
    hostname: String,
}

impl HostsEntry {
    pub fn new(ip: impl Into<String>, hostname: impl Into<String>) -> Result<Self, DomainError> {
        let ip = ip.into();
        let hostname = hostname.into();
        Self::validate_ip(&ip)?;
        Self::validate_hostname(&hostname)?;
        Ok(Self { ip, hostname })
    }

    fn validate_ip(ip: &str) -> Result<(), DomainError> {
        ip.parse::<IpAddr>()
            .map_err(|_| DomainError::InvalidHostsEntry(format!("invalid IP address: {ip}")))?;
        Ok(())
    }

    fn validate_hostname(hostname: &str) -> Result<(), DomainError> {
        if hostname.is_empty() {
            return Err(DomainError::InvalidHostsEntry(
                "hostname cannot be empty".to_string(),
            ));
        }
        if hostname.len() > 253 {
            return Err(DomainError::InvalidHostsEntry(
                "hostname too long".to_string(),
            ));
        }
        let valid = hostname.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        });
        if !valid {
            return Err(DomainError::InvalidHostsEntry(format!(
                "invalid hostname: {hostname}"
            )));
        }
        Ok(())
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }
}

impl fmt::Display for HostsEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ip, self.hostname)
    }
}
