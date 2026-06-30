use std::io::Write;

use librrmj::ai::Difficulty;
use librrmj::rules::RulesProfileId;
use tempfile::NamedTempFile;

use clap::Parser;

use crate::cli::Cli;
use crate::config::{FileConfig, parse_difficulty};
use crate::settings::Settings;
use crate::utils::{DEFAULT_CPU_MS, DEFAULT_RESPONSE_MS, DEFAULT_TURN_MS};

#[test]
fn default_settings_values() {
    let settings = Settings::default();
    assert_eq!(settings.theme, "default");
    assert_eq!(settings.rules_profile, RulesProfileId::Standard);
    assert_eq!(settings.rules_config().profile, RulesProfileId::Standard);
    assert_eq!(settings.default_difficulty, Difficulty::Medium);
    assert_eq!(settings.human_seat, 0);
    assert_eq!(settings.cpu_step_delay_ms, DEFAULT_CPU_MS);
    assert_eq!(settings.turn_timer_ms, DEFAULT_TURN_MS);
    assert_eq!(settings.response_timer_ms, DEFAULT_RESPONSE_MS);
    assert_eq!(settings.log_level, "info");
}

#[test]
fn resolve_merges_config_file() {
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
log_level = "debug"
"#
    )
    .unwrap();
    let cli = Cli::try_parse_from(["rrmj-tui", "--config", file.path().to_str().unwrap()]).unwrap();
    let settings = Settings::resolve(&cli).unwrap();
    assert_eq!(settings.theme, "high-contrast");
    assert_eq!(settings.rules_profile, RulesProfileId::Standard);
    assert_eq!(settings.default_difficulty, Difficulty::Hard);
    assert_eq!(settings.human_seat, 2);
    assert_eq!(settings.cpu_step_delay_ms, 500);
    assert_eq!(settings.turn_timer_ms, 15_000);
    assert_eq!(settings.response_timer_ms, 3000);
    assert_eq!(settings.log_level, "debug");
}

#[test]
fn resolve_without_config_uses_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.toml");
    let cli = Cli::try_parse_from(["rrmj-tui", "--config", missing.to_str().unwrap()]).unwrap();
    let settings = Settings::resolve(&cli).unwrap();
    assert_eq!(settings.theme, "default");
    assert_eq!(settings.default_difficulty, Difficulty::Medium);
}

#[test]
fn config_roundtrip_save_load() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();
    let settings = Settings {
        config_path: path.clone(),
        ..Settings::default()
    };
    settings.save_config().unwrap();
    let loaded = FileConfig::from_file(&path).unwrap();
    assert_eq!(loaded.theme.as_deref(), Some("default"));
    assert_eq!(loaded.rules_profile, Some(RulesProfileId::Standard));
    assert_eq!(loaded.default_difficulty, Some(Difficulty::Medium));
}

#[test]
fn invalid_difficulty_errors() {
    let err = parse_difficulty("insane").unwrap_err();
    assert!(err.contains("unknown difficulty"));
}
