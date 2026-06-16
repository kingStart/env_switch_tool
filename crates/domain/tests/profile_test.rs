use envtools_domain::model::profile::Profile;

#[test]
fn create_profile() {
    let profile = Profile::new("fullstack-dev", "Full stack dev env").unwrap();
    assert_eq!(profile.name(), "fullstack-dev");
    assert_eq!(profile.description(), "Full stack dev env");
    assert!(profile.group_names().is_empty());
}

#[test]
fn add_group_to_profile() {
    let mut profile = Profile::new("test", "").unwrap();
    profile.add_group("nodejs-env".to_string());
    profile.add_group("local-dns".to_string());
    assert_eq!(profile.group_names(), &["nodejs-env", "local-dns"]);
}

#[test]
fn add_duplicate_group_is_idempotent() {
    let mut profile = Profile::new("test", "").unwrap();
    profile.add_group("g1".to_string());
    profile.add_group("g1".to_string());
    assert_eq!(profile.group_names().len(), 1);
}

#[test]
fn remove_group_from_profile() {
    let mut profile = Profile::new("test", "").unwrap();
    profile.add_group("g1".to_string());
    profile.add_group("g2".to_string());
    profile.remove_group("g1");
    assert_eq!(profile.group_names(), &["g2"]);
}

#[test]
fn from_state_restores_groups() {
    let profile = Profile::from_state(
        "restored".to_string(),
        "desc".to_string(),
        vec!["a".to_string(), "b".to_string()],
    );
    assert_eq!(profile.group_names().len(), 2);
}
