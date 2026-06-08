use super::{RulesConfig, RulesProfileId};

#[test]
fn standard_profile_defaults() {
    let config = RulesConfig::standard();
    assert_eq!(config.profile, RulesProfileId::Standard);
    assert_eq!(config.starting_points, 25_000);
    assert!(config.aka_dora);
    assert!(!config.kiriage);
}

#[test]
fn default_for_matches_standard() {
    assert_eq!(
        RulesConfig::standard(),
        RulesConfig::default_for(RulesProfileId::Standard),
    );
}
