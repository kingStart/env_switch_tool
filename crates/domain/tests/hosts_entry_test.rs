use envtools_domain::error::DomainError;
use envtools_domain::model::hosts_entry::HostsEntry;

#[test]
fn valid_ipv4_entry() {
    let entry = HostsEntry::new("127.0.0.1", "api.local").unwrap();
    assert_eq!(entry.ip(), "127.0.0.1");
    assert_eq!(entry.hostname(), "api.local");
    assert_eq!(entry.to_string(), "127.0.0.1 api.local");
}

#[test]
fn valid_ipv6_entry() {
    let entry = HostsEntry::new("::1", "localhost").unwrap();
    assert_eq!(entry.ip(), "::1");
    assert_eq!(entry.hostname(), "localhost");
}

#[test]
fn invalid_ip_rejected() {
    let result = HostsEntry::new("not-an-ip", "test.local");
    assert!(matches!(result, Err(DomainError::InvalidHostsEntry(_))));
}

#[test]
fn empty_hostname_rejected() {
    let result = HostsEntry::new("127.0.0.1", "");
    assert!(matches!(result, Err(DomainError::InvalidHostsEntry(_))));
}

#[test]
fn hostname_with_invalid_chars_rejected() {
    let result = HostsEntry::new("127.0.0.1", "host_name.local");
    assert!(matches!(result, Err(DomainError::InvalidHostsEntry(_))));
}

#[test]
fn hostname_starting_with_hyphen_rejected() {
    let result = HostsEntry::new("127.0.0.1", "-bad.local");
    assert!(matches!(result, Err(DomainError::InvalidHostsEntry(_))));
}
