use super::{RulesConfig, RulesProfileId};

#[test]
fn standard_profile_defaults() {
    let config = RulesConfig::standard();
    assert_eq!(config.profile, RulesProfileId::Standard);
    assert_eq!(config.starting_points, 25_000);
    assert!(config.aka_dora);
    assert!(!config.kiriage);
    assert!(config.abortive_nine_terminals);
    assert_eq!(config.game_length, crate::game::GameLength::Hanchan);
    assert!(config.target_score.is_none());
}

#[test]
fn profile_id_parse_and_cycle() {
    assert_eq!(
        RulesProfileId::parse("standard").unwrap(),
        RulesProfileId::Standard
    );
    assert_eq!(RulesProfileId::Standard.next(), RulesProfileId::Standard);
    assert!(RulesProfileId::parse("mcr").is_err());
}

#[test]
fn default_for_matches_standard() {
    assert_eq!(
        RulesConfig::standard(),
        RulesConfig::default_for(RulesProfileId::Standard),
    );
}
