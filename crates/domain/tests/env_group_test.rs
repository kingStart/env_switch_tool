use envtools_domain::event::DomainEvent;
use envtools_domain::model::env_variable::EnvVariable;
use envtools_domain::model::group_kind::GroupKind;
use envtools_domain::model::managed_group::ManagedGroup;
use envtools_domain::model::priority::Priority;

type EnvGroup = ManagedGroup;

#[test]
fn new_group_is_inactive_and_emits_created_event() {
    let mut group = EnvGroup::new("java-dev", "Java 17 开发环境");
    assert!(!group.is_active());
    assert_eq!(group.name(), "java-dev");
    assert_eq!(group.description(), "Java 17 开发环境");
    assert_eq!(group.variables().len(), 0);

    let events = group.take_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], DomainEvent::GroupCreated { name } if name == "java-dev"));
}

#[test]
fn with_priority_sets_correct_priority() {
    let group = EnvGroup::with_priority("prod", "Production", 10);
    assert_eq!(group.priority(), Priority::new(10));
}

#[test]
fn enable_emits_event_and_changes_state() {
    let mut group = EnvGroup::new("test", "");
    let events = group.enable();

    assert!(group.is_active());
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], DomainEvent::GroupEnabled { name } if name == "test"));
}

#[test]
fn enable_already_active_group_emits_nothing() {
    let mut group = EnvGroup::new("test", "");
    group.enable();
    let events = group.enable();
    assert!(events.is_empty());
}

#[test]
fn disable_emits_event_and_changes_state() {
    let mut group = EnvGroup::new("test", "");
    group.enable();
    let events = group.disable();

    assert!(!group.is_active());
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], DomainEvent::GroupDisabled { name } if name == "test"));
}

#[test]
fn disable_already_inactive_group_emits_nothing() {
    let mut group = EnvGroup::new("test", "");
    let events = group.disable();
    assert!(events.is_empty());
}

#[test]
fn add_variable_stores_and_emits_event() {
    let mut group = EnvGroup::new("test", "");
    let var = EnvVariable::new("JAVA_HOME", "/usr/lib/jvm/java-17").unwrap();
    let events = group.add_variable(var).unwrap();

    assert_eq!(group.variables().len(), 1);
    assert_eq!(group.variables()[0].key(), "JAVA_HOME");
    assert_eq!(events.len(), 1);
    assert!(
        matches!(&events[0], DomainEvent::VariableAdded { group_name, .. } if group_name == "test")
    );
}

#[test]
fn add_duplicate_key_updates_instead() {
    let mut group = EnvGroup::new("test", "");
    group
        .add_variable(EnvVariable::new("KEY", "old").unwrap())
        .unwrap();
    let events = group
        .add_variable(EnvVariable::new("KEY", "new").unwrap())
        .unwrap();

    assert_eq!(group.variables().len(), 1);
    assert_eq!(group.get_variable("KEY").unwrap().value(), "new");
    assert!(matches!(&events[0], DomainEvent::VariableUpdated { .. }));
}

#[test]
fn remove_variable_removes_and_emits_event() {
    let mut group = EnvGroup::new("test", "");
    group
        .add_variable(EnvVariable::new("KEY", "val").unwrap())
        .unwrap();
    let events = group.remove_variable("KEY").unwrap();

    assert_eq!(group.variables().len(), 0);
    assert_eq!(events.len(), 1);
    assert!(
        matches!(&events[0], DomainEvent::VariableRemoved { group_name, key } if group_name == "test" && key == "KEY")
    );
}

#[test]
fn remove_nonexistent_variable_returns_empty() {
    let mut group = EnvGroup::new("test", "");
    let events = group.remove_variable("NOPE").unwrap();
    assert!(events.is_empty());
}

#[test]
fn from_state_does_not_emit_events() {
    let var = EnvVariable::new("KEY", "val").unwrap();
    let mut group = EnvGroup::from_state(
        "restored".to_string(),
        "desc".to_string(),
        GroupKind::Env,
        vec![var],
        Vec::new(),
        true,
        Priority::new(5),
    );

    assert!(group.is_active());
    assert_eq!(group.variables().len(), 1);
    assert!(group.take_events().is_empty());
}
