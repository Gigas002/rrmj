mod paths;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;

use librrmj::ai::Difficulty;
use librrmj::rules::{RulesConfig, RulesProfileId, RulesRegistry};

use crate::error::AppError;

pub use paths::{config_dir, config_path, keybinds_path, recordings_dir};

/// User-facing settings loaded from `config.toml` or baked-in defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub theme: String,
    pub rules_profile: RulesProfileId,
    pub default_difficulty: Difficulty,
    pub human_seat: usize,
    /// Pause between CPU decisions (presentation).
    pub cpu_step_delay_ms: u64,
    /// Per-turn thinking limit for draw/discard; `0` = unlimited.
    pub turn_timer_ms: u64,
    /// Reaction window for chi/pon/ron/pass; `0` = unlimited (pass-only still instant).
    pub response_timer_ms: u64,
    pub recordings_dir: Option<std::path::PathBuf>,
    #[cfg(feature = "debug-menu")]
    pub scenarios_dir: Option<std::path::PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".into(),
            rules_profile: RulesProfileId::Standard,
            default_difficulty: Difficulty::Medium,
            human_seat: 0,
            cpu_step_delay_ms: crate::timers::DEFAULT_CPU_MS,
            turn_timer_ms: crate::timers::DEFAULT_TURN_MS,
            response_timer_ms: crate::timers::DEFAULT_RESPONSE_MS,
            recordings_dir: None,
            #[cfg(feature = "debug-menu")]
            scenarios_dir: None,
        }
    }
}

impl AppConfig {
    pub fn load(path: Option<&Path>) -> Result<Self, AppError> {
        let path = path.map(Path::to_path_buf).unwrap_or_else(config_path);

        if path.exists() {
            Self::from_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, AppError> {
        let text = fs::read_to_string(path).map_err(AppError::Terminal)?;
        let table: toml::Table = toml::from_str(&text).map_err(|err| AppError::Config {
            path: path.to_path_buf(),
            detail: err.to_string(),
        })?;

        let mut cfg = Self::default();
        if let Some(value) = table.get("theme").and_then(|v| v.as_str()) {
            cfg.theme = value.to_string();
        }
        if let Some(value) = table.get("rules_profile").and_then(|v| v.as_str()) {
            cfg.rules_profile =
                RulesProfileId::parse(value).map_err(|detail| AppError::Config {
                    path: path.to_path_buf(),
                    detail,
                })?;
        }
        if let Err(err) = RulesRegistry::get(cfg.rules_profile) {
            return Err(AppError::Config {
                path: path.to_path_buf(),
                detail: err.to_string(),
            });
        }
        if let Some(value) = table.get("default_difficulty").and_then(|v| v.as_str()) {
            cfg.default_difficulty =
                parse_difficulty(value).map_err(|detail| AppError::Config {
                    path: path.to_path_buf(),
                    detail,
                })?;
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
            cfg.human_seat = seat;
        }
        if let Some(value) = table.get("cpu_step_delay_ms").and_then(|v| v.as_integer()) {
            if value < 0 {
                return Err(AppError::Config {
                    path: path.to_path_buf(),
                    detail: "cpu_step_delay_ms must be >= 0".into(),
                });
            }
            cfg.cpu_step_delay_ms = crate::timers::normalize_cpu(u64::try_from(value).unwrap_or(0));
        }
        if let Some(value) = table.get("turn_timer_ms").and_then(|v| v.as_integer()) {
            if value < 0 {
                return Err(AppError::Config {
                    path: path.to_path_buf(),
                    detail: "turn_timer_ms must be >= 0".into(),
                });
            }
            cfg.turn_timer_ms = crate::timers::normalize_turn(u64::try_from(value).unwrap_or(0));
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
            cfg.response_timer_ms =
                crate::timers::normalize_response(u64::try_from(value).unwrap_or(0));
        }
        if let Some(value) = table
            .get("recordings_dir")
            .or_else(|| table.get("saves_dir"))
            .and_then(|v| v.as_str())
        {
            cfg.recordings_dir = Some(value.into());
        }
        #[cfg(feature = "debug-menu")]
        if let Some(value) = table.get("scenarios_dir").and_then(|v| v.as_str()) {
            cfg.scenarios_dir = Some(value.into());
        }

        Ok(cfg)
    }

    pub fn resolved_recordings_dir(&self) -> std::path::PathBuf {
        self.recordings_dir.clone().unwrap_or_else(recordings_dir)
    }

    /// Engine rules for new games (tunables come from [`RulesConfig::default_for`]).
    pub fn rules_config(&self) -> RulesConfig {
        RulesConfig::default_for(self.rules_profile)
    }

    #[cfg(feature = "debug-menu")]
    pub fn resolved_scenarios_dir(&self) -> std::path::PathBuf {
        crate::scenarios::resolve_scenarios_dir(self.scenarios_dir.as_deref())
    }

    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(AppError::Terminal)?;
        }
        let body = self.to_toml();
        fs::write(path, body).map_err(AppError::Terminal)
    }

    fn to_toml(&self) -> String {
        format!(
            r#"# rrmj TUI settings — see README for details.

theme = "{theme}"
rules_profile = "{rules_profile}"
default_difficulty = "{difficulty}"
human_seat = {human_seat}
cpu_step_delay_ms = {cpu_step_delay_ms}
turn_timer_ms = {turn_timer_ms}
response_timer_ms = {response_timer_ms}
"#,
            theme = self.theme,
            rules_profile = self.rules_profile.as_str(),
            difficulty = difficulty_name(self.default_difficulty),
            human_seat = self.human_seat,
            cpu_step_delay_ms = self.cpu_step_delay_ms,
            turn_timer_ms = self.turn_timer_ms,
            response_timer_ms = self.response_timer_ms,
        )
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
