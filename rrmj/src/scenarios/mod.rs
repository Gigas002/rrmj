#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use librrmj::replay::GameRecording;

use crate::error::AppError;
use crate::save::resolve_user_path;

#[derive(Debug, Clone)]
pub struct ScenarioEntry {
    pub path: PathBuf,
    pub id: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Repo CI fixtures (`examples/scenarios`) — debug menu and tests only.
#[cfg(any(test, feature = "debug-menu"))]
pub fn bundled_debug_scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/scenarios")
}

pub fn list_scenarios(dir: &Path) -> Result<Vec<ScenarioEntry>, AppError> {
    let read_dir = match fs::read_dir(dir) {
        Ok(dir) => dir,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(AppError::Terminal(err)),
    };

    let mut entries = Vec::new();
    for entry in read_dir {
        let entry = entry.map_err(AppError::Terminal)?;
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(AppError::Terminal)?;
        let recording = GameRecording::from_json(&text).map_err(AppError::Engine)?;
        entries.push(scenario_entry(path, &recording));
    }

    entries.sort_by(|a, b| {
        let tag_a = a.tags.first().map(String::as_str).unwrap_or("");
        let tag_b = b.tags.first().map(String::as_str).unwrap_or("");
        tag_a
            .cmp(tag_b)
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(entries)
}

pub fn read_scenario(path: &Path) -> Result<GameRecording, AppError> {
    let text = fs::read_to_string(path).map_err(AppError::Terminal)?;
    GameRecording::from_json(&text).map_err(AppError::Engine)
}

fn scenario_entry(path: PathBuf, recording: &GameRecording) -> ScenarioEntry {
    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    ScenarioEntry {
        title: recording.meta.title.clone().unwrap_or_else(|| id.clone()),
        description: recording.meta.description.clone().unwrap_or_default(),
        tags: recording.meta.tags.clone(),
        id,
        path,
    }
}

/// Resolve a user-typed path and load a scenario from anywhere on disk.
pub fn load_scenario_from_path(path: &str) -> Result<(ScenarioEntry, GameRecording), AppError> {
    let path = resolve_scenario_path(path);
    let recording = read_scenario(&path)?;
    recording.validate().map_err(AppError::Engine)?;
    let entry = scenario_entry(path, &recording);
    Ok((entry, recording))
}

fn resolve_scenario_path(path: &str) -> PathBuf {
    let resolved = resolve_user_path(path.trim());
    if resolved.is_file() {
        return resolved;
    }
    if resolved.extension().is_none() {
        let with_json = resolved.with_extension("json");
        if with_json.is_file() {
            return with_json;
        }
    }
    resolved
}

pub fn all_tags(entries: &[ScenarioEntry]) -> Vec<String> {
    let mut tags: Vec<String> = entries
        .iter()
        .flat_map(|e| e.tags.iter().cloned())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

pub fn filter_by_tag<'a>(
    entries: &'a [ScenarioEntry],
    tag: Option<&str>,
) -> Vec<&'a ScenarioEntry> {
    let Some(tag) = tag else {
        return entries.iter().collect();
    };
    entries
        .iter()
        .filter(|e| e.tags.iter().any(|t| t == tag))
        .collect()
}
