use envtools_domain::error::DomainError;
use envtools_domain::model::group_kind::GroupKind;
use envtools_domain::model::hosts_entry::HostsEntry;
use envtools_domain::model::managed_group::ManagedGroup;

#[test]
fn new_group_defaults_to_env_kind() {
    let group = ManagedGroup::new("test", "desc");
    assert_eq!(group.kind(), GroupKind::Env);
    assert!(group.hosts_entries().is_empty());
}

#[test]
fn new_hosts_group_has_hosts_kind() {
    let group = ManagedGroup::new_hosts("dns-local", "local dns");
    assert_eq!(group.kind(), GroupKind::Hosts);
    assert!(group.variables().is_empty());
}

#[test]
fn cannot_add_variable_to_hosts_group() {
    let mut group = ManagedGroup::new_hosts("dns", "");
    let var = envtools_domain::model::env_variable::EnvVariable::new("KEY", "val").unwrap();
    let result = group.add_variable(var);
    assert!(matches!(result, Err(DomainError::InvalidVariableKey(_))));
}

#[test]
fn cannot_add_hosts_entry_to_env_group() {
    let mut group = ManagedGroup::new("env", "");
    let entry = HostsEntry::new("127.0.0.1", "test.local").unwrap();
    let result = group.add_hosts_entry(entry);
    assert!(matches!(result, Err(DomainError::InvalidHostsEntry(_))));
}

#[test]
fn add_and_remove_hosts_entry() {
    let mut group = ManagedGroup::new_hosts("dns", "");
    let entry = HostsEntry::new("10.0.0.1", "api.dev").unwrap();
    group.add_hosts_entry(entry).unwrap();
    assert_eq!(group.hosts_entries().len(), 1);

    group.remove_hosts_entry("api.dev").unwrap();
    assert_eq!(group.hosts_entries().len(), 0);
}

#[test]
fn duplicate_hostname_updates_ip() {
    let mut group = ManagedGroup::new_hosts("dns", "");
    group
        .add_hosts_entry(HostsEntry::new("10.0.0.1", "api.dev").unwrap())
        .unwrap();
    group
        .add_hosts_entry(HostsEntry::new("192.168.1.1", "api.dev").unwrap())
        .unwrap();

    assert_eq!(group.hosts_entries().len(), 1);
    assert_eq!(group.hosts_entries()[0].ip(), "192.168.1.1");
}
