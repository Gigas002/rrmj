mod paths;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;

use librrmj::ai::Difficulty;

use crate::error::AppError;

pub use paths::{config_dir, config_path, keybinds_path};

/// User-facing settings loaded from `config.toml` or baked-in defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub theme: String,
    pub default_difficulty: Difficulty,
    pub human_seat: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".into(),
            default_difficulty: Difficulty::Medium,
            human_seat: 0,
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

        Ok(cfg)
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
default_difficulty = "{difficulty}"
human_seat = {human_seat}
"#,
            theme = self.theme,
            difficulty = difficulty_name(self.default_difficulty),
            human_seat = self.human_seat,
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
