use std::io::Write;

use librrmj::ai::Difficulty;
use librrmj::rules::RulesProfileId;
use tempfile::NamedTempFile;

use crate::config::FileConfig;

#[test]
fn parse_config_file() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
theme = "high-contrast"
rules_profile = "standard"
default_difficulty = "hard"
human_seat = 2
cpu_step_delay_ms = 500
turn_timer_ms = 15000
response_timer_ms = 3000
"#
    )
    .unwrap();
    let cfg = FileConfig::from_file(file.path()).unwrap();
    assert_eq!(cfg.theme.as_deref(), Some("high-contrast"));
    assert_eq!(cfg.rules_profile, Some(RulesProfileId::Standard));
    assert_eq!(cfg.default_difficulty, Some(Difficulty::Hard));
    assert_eq!(cfg.human_seat, Some(2));
    assert_eq!(cfg.cpu_step_delay_ms, Some(500));
    assert_eq!(cfg.turn_timer_ms, Some(15_000));
    assert_eq!(cfg.response_timer_ms, Some(3000));
}

#[test]
fn legacy_reaction_pass_delay_alias() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
reaction_pass_delay_ms = 3000
"#
    )
    .unwrap();
    let cfg = FileConfig::from_file(file.path()).unwrap();
    assert_eq!(cfg.response_timer_ms, Some(3000));
}

#[test]
fn invalid_rules_profile_errors() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, r#"rules_profile = "mcr""#).unwrap();
    let err = FileConfig::from_file(file.path()).unwrap_err().to_string();
    assert!(err.contains("unknown rules profile"));
}

#[test]
fn invalid_difficulty_errors() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, r#"default_difficulty = "insane""#).unwrap();
    let err = FileConfig::from_file(file.path()).unwrap_err().to_string();
    assert!(err.contains("unknown difficulty"));
}
