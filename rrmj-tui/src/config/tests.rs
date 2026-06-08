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
"#
    )
    .unwrap();
    let cfg = AppConfig::from_file(file.path()).unwrap();
    assert_eq!(cfg.theme, "high-contrast");
    assert_eq!(cfg.default_difficulty, Difficulty::Hard);
    assert_eq!(cfg.human_seat, 2);
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
}
