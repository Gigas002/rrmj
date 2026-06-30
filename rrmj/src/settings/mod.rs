use std::fs;
use std::path::{Path, PathBuf};

use librrmj::ai::Difficulty;
use librrmj::rules::{RulesConfig, RulesProfileId, RulesRegistry};

use crate::cli::Cli;
use crate::config::{
    FileConfig, config_path, difficulty_name, keybinds_path, recordings_dir, scenarios_dir,
};
use crate::error::AppError;
use crate::input::Keybinds;
use crate::utils::{
    DEFAULT_CPU_MS, DEFAULT_RESPONSE_MS, DEFAULT_TURN_MS, normalize_cpu, normalize_response,
    normalize_turn,
};

#[cfg(test)]
mod tests;

/// Fully resolved runtime settings — the only configuration type below this boundary.
#[derive(Debug, Clone)]
pub struct Settings {
    pub theme: String,
    pub rules_profile: RulesProfileId,
    pub default_difficulty: Difficulty,
    pub human_seat: usize,
    pub cpu_step_delay_ms: u64,
    pub turn_timer_ms: u64,
    pub response_timer_ms: u64,
    pub recordings_dir: Option<PathBuf>,
    pub scenarios_dir: Option<PathBuf>,
    pub log_level: String,
    pub config_path: PathBuf,
    pub keybinds_path: PathBuf,
    pub keybinds: Keybinds,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "default".into(),
            rules_profile: RulesProfileId::Standard,
            default_difficulty: Difficulty::Medium,
            human_seat: 0,
            cpu_step_delay_ms: DEFAULT_CPU_MS,
            turn_timer_ms: DEFAULT_TURN_MS,
            response_timer_ms: DEFAULT_RESPONSE_MS,
            recordings_dir: None,
            scenarios_dir: None,
            log_level: "info".into(),
            config_path: config_path(),
            keybinds_path: keybinds_path(),
            keybinds: Keybinds::default_map(),
        }
    }
}

impl Settings {
    /// Merge CLI paths, optional config file, and built-in defaults.
    pub fn resolve(cli: &Cli) -> Result<Self, AppError> {
        let config_path = cli.config.clone().unwrap_or_else(config_path);
        let keybinds_path = cli.keybinds.clone().unwrap_or_else(keybinds_path);

        let mut settings = Self {
            config_path: config_path.clone(),
            keybinds_path: keybinds_path.clone(),
            ..Self::default()
        };

        if config_path.exists() {
            let file = FileConfig::from_file(&config_path)?;
            settings.apply_file(&file, &config_path)?;
        }

        settings.keybinds = Keybinds::load(Some(&keybinds_path))?;
        Ok(settings)
    }

    fn apply_file(&mut self, file: &FileConfig, path: &Path) -> Result<(), AppError> {
        if let Some(theme) = &file.theme {
            self.theme = theme.clone();
        }
        if let Some(profile) = file.rules_profile {
            RulesRegistry::get(profile).map_err(|err| AppError::Config {
                path: path.to_path_buf(),
                detail: err.to_string(),
            })?;
            self.rules_profile = profile;
        }
        if let Some(difficulty) = file.default_difficulty {
            self.default_difficulty = difficulty;
        }
        if let Some(seat) = file.human_seat {
            self.human_seat = seat;
        }
        if let Some(ms) = file.cpu_step_delay_ms {
            self.cpu_step_delay_ms = normalize_cpu(ms);
        }
        if let Some(ms) = file.turn_timer_ms {
            self.turn_timer_ms = normalize_turn(ms);
        }
        if let Some(ms) = file.response_timer_ms {
            self.response_timer_ms = normalize_response(ms);
        }
        if let Some(dir) = &file.recordings_dir {
            self.recordings_dir = Some(dir.clone());
        }
        if let Some(dir) = &file.scenarios_dir {
            self.scenarios_dir = Some(dir.clone());
        }
        if let Some(level) = &file.log_level {
            self.log_level = level.clone();
        }
        Ok(())
    }

    pub fn resolved_recordings_dir(&self) -> PathBuf {
        self.recordings_dir.clone().unwrap_or_else(recordings_dir)
    }

    pub fn resolved_scenarios_dir(&self) -> PathBuf {
        self.scenarios_dir.clone().unwrap_or_else(scenarios_dir)
    }

    pub fn rules_config(&self) -> RulesConfig {
        RulesConfig::default_for(self.rules_profile)
    }

    pub fn log_level(&self) -> &str {
        &self.log_level
    }

    pub fn save_config(&self) -> Result<(), AppError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(AppError::Terminal)?;
        }
        let body = self.to_toml();
        fs::write(&self.config_path, body).map_err(AppError::Terminal)
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
log_level = "{log_level}"
"#,
            theme = self.theme,
            rules_profile = self.rules_profile.as_str(),
            difficulty = difficulty_name(self.default_difficulty),
            human_seat = self.human_seat,
            cpu_step_delay_ms = self.cpu_step_delay_ms,
            turn_timer_ms = self.turn_timer_ms,
            response_timer_ms = self.response_timer_ms,
            log_level = self.log_level,
        )
    }
}
