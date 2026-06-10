#![cfg(feature = "debug-menu")]

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use librrmj::replay::MatchRecording;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ScenarioEntry {
    pub path: PathBuf,
    pub id: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}

pub fn default_scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/scenarios")
}

pub fn resolve_scenarios_dir(configured: Option<&Path>) -> PathBuf {
    if let Some(dir) = configured {
        return dir.to_path_buf();
    }
    let dev = default_scenarios_dir();
    if dev.exists() {
        return dev;
    }
    PathBuf::from("examples/scenarios")
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
        let recording = MatchRecording::from_json(&text).map_err(AppError::Engine)?;
        let id = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        entries.push(ScenarioEntry {
            title: recording.meta.title.clone().unwrap_or_else(|| id.clone()),
            description: recording.meta.description.clone().unwrap_or_default(),
            tags: recording.meta.tags.clone(),
            id,
            path,
        });
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

pub fn read_scenario(path: &Path) -> Result<MatchRecording, AppError> {
    let text = fs::read_to_string(path).map_err(AppError::Terminal)?;
    MatchRecording::from_json(&text).map_err(AppError::Engine)
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
