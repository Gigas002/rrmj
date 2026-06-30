mod paths;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use librrmj::ai::Difficulty;
use librrmj::rules::RulesProfileId;

use crate::error::AppError;

pub use paths::{config_dir, config_path, keybinds_path, recordings_dir, scenarios_dir};

/// Deserialized TOML fields — only keys present in the file are set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileConfig {
    pub theme: Option<String>,
    pub rules_profile: Option<RulesProfileId>,
    pub default_difficulty: Option<Difficulty>,
    pub human_seat: Option<usize>,
    pub cpu_step_delay_ms: Option<u64>,
    pub turn_timer_ms: Option<u64>,
    pub response_timer_ms: Option<u64>,
    pub recordings_dir: Option<PathBuf>,
    pub scenarios_dir: Option<PathBuf>,
    pub log_level: Option<String>,
}

impl FileConfig {
    /// Read and deserialize a config file. Errors if the path is missing or invalid.
    pub fn from_file(path: &Path) -> Result<Self, AppError> {
        let text = fs::read_to_string(path).map_err(AppError::Terminal)?;
        let table: toml::Table = toml::from_str(&text).map_err(|err| AppError::Config {
            path: path.to_path_buf(),
            detail: err.to_string(),
        })?;

        let mut cfg = Self::default();
        if let Some(value) = table.get("theme").and_then(|v| v.as_str()) {
            cfg.theme = Some(value.to_string());
        }
        if let Some(value) = table.get("rules_profile").and_then(|v| v.as_str()) {
            cfg.rules_profile =
                Some(
                    RulesProfileId::parse(value).map_err(|detail| AppError::Config {
                        path: path.to_path_buf(),
                        detail,
                    })?,
                );
        }
        if let Some(value) = table.get("default_difficulty").and_then(|v| v.as_str()) {
            cfg.default_difficulty =
                Some(parse_difficulty(value).map_err(|detail| AppError::Config {
                    path: path.to_path_buf(),
                    detail,
                })?);
        }
        if let Some(value) = table.get("human_seat").and_then(|v| v.as_integer()) {
            let seat = usize::try_from(value).map_err(|_| AppError::Config {
                path: path.to_path_buf(),
                detail: "human_seat must be 0–3".into(),
            })?;
            if seat > 3 {
                return Err(AppError::Config {
                    path: path.to_path_buf(),
                    detail: "human_seat must be 0–3".into(),
                });
            }
            cfg.human_seat = Some(seat);
        }
        if let Some(value) = table.get("cpu_step_delay_ms").and_then(|v| v.as_integer()) {
            if value < 0 {
                return Err(AppError::Config {
                    path: path.to_path_buf(),
                    detail: "cpu_step_delay_ms must be >= 0".into(),
                });
            }
            cfg.cpu_step_delay_ms = Some(u64::try_from(value).unwrap_or(0));
        }
        if let Some(value) = table.get("turn_timer_ms").and_then(|v| v.as_integer()) {
            if value < 0 {
                return Err(AppError::Config {
                    path: path.to_path_buf(),
                    detail: "turn_timer_ms must be >= 0".into(),
                });
            }
            cfg.turn_timer_ms = Some(u64::try_from(value).unwrap_or(0));
        }
        if let Some(value) = table
            .get("response_timer_ms")
            .or_else(|| table.get("reaction_pass_delay_ms"))
            .and_then(|v| v.as_integer())
        {
            if value < 0 {
                return Err(AppError::Config {
                    path: path.to_path_buf(),
                    detail: "response_timer_ms must be >= 0".into(),
                });
            }
            cfg.response_timer_ms = Some(u64::try_from(value).unwrap_or(0));
        }
        if let Some(value) = table
            .get("recordings_dir")
            .or_else(|| table.get("saves_dir"))
            .and_then(|v| v.as_str())
        {
            cfg.recordings_dir = Some(value.into());
        }
        if let Some(value) = table.get("scenarios_dir").and_then(|v| v.as_str()) {
            cfg.scenarios_dir = Some(value.into());
        }
        if let Some(value) = table.get("log_level").and_then(|v| v.as_str()) {
            cfg.log_level = Some(value.to_string());
        }

        Ok(cfg)
    }
}

pub fn parse_difficulty(name: &str) -> Result<Difficulty, String> {
    match name.to_ascii_lowercase().as_str() {
        "easy" => Ok(Difficulty::Easy),
        "medium" => Ok(Difficulty::Medium),
        "hard" => Ok(Difficulty::Hard),
        other => Err(format!(
            "unknown difficulty '{other}' (expected easy, medium, or hard)"
        )),
    }
}

pub fn difficulty_name(d: Difficulty) -> &'static str {
    match d {
        Difficulty::Easy => "easy",
        Difficulty::Medium => "medium",
        Difficulty::Hard => "hard",
    }
}

pub fn theme_names() -> &'static [&'static str] {
    &["default", "high-contrast"]
}

pub fn cycle_theme(current: &str) -> String {
    let names = theme_names();
    let idx = names.iter().position(|n| *n == current).unwrap_or(0);
    names[(idx + 1) % names.len()].to_string()
}
