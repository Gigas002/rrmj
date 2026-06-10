use std::io::Write;

use librrmj::ai::Difficulty;
use tempfile::NamedTempFile;

use crate::config::{AppConfig, parse_difficulty};

#[test]
fn default_config_values() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.theme, "default");
    assert_eq!(cfg.default_difficulty, Difficulty::Medium);
    assert_eq!(cfg.human_seat, 0);
    assert_eq!(cfg.cpu_step_delay_ms, crate::timers::DEFAULT_CPU_MS);
    assert_eq!(cfg.turn_timer_ms, crate::timers::DEFAULT_TURN_MS);
    assert_eq!(cfg.response_timer_ms, crate::timers::DEFAULT_RESPONSE_MS);
}

#[test]
fn parse_config_file() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
theme = "high-contrast"
default_difficulty = "hard"
human_seat = 2
cpu_step_delay_ms = 500
turn_timer_ms = 15000
response_timer_ms = 3000
"#
    )
    .unwrap();
    let cfg = AppConfig::from_file(file.path()).unwrap();
    assert_eq!(cfg.theme, "high-contrast");
    assert_eq!(cfg.default_difficulty, Difficulty::Hard);
    assert_eq!(cfg.human_seat, 2);
    assert_eq!(cfg.cpu_step_delay_ms, 500);
    assert_eq!(cfg.turn_timer_ms, 15_000);
    assert_eq!(cfg.response_timer_ms, 3000);
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
    let cfg = AppConfig::from_file(file.path()).unwrap();
    assert_eq!(cfg.response_timer_ms, 3000);
}

#[test]
fn invalid_difficulty_errors() {
    let err = parse_difficulty("insane").unwrap_err();
    assert!(err.contains("unknown difficulty"));
}

#[test]
fn config_roundtrip_save_load() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();
    let cfg = AppConfig::default();
    cfg.save(&path).unwrap();
    let loaded = AppConfig::from_file(&path).unwrap();
    assert_eq!(cfg.theme, loaded.theme);
    assert_eq!(cfg.default_difficulty, loaded.default_difficulty);
    assert_eq!(cfg.cpu_step_delay_ms, loaded.cpu_step_delay_ms);
    assert_eq!(cfg.turn_timer_ms, loaded.turn_timer_ms);
    assert_eq!(cfg.response_timer_ms, loaded.response_timer_ms);
}
