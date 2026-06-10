#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use librrmj::replay::{MatchRecording, MatchStatus};

use crate::error::AppError;

/// Client-owned directory for all match recordings (in-progress and finished).
#[derive(Debug, Clone)]
pub struct SavePaths {
    pub recordings_dir: PathBuf,
}

impl SavePaths {
    pub fn ensure_dir(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.recordings_dir).map_err(AppError::Terminal)
    }

    pub fn recording_path(&self, recording_id: &str) -> PathBuf {
        self.recordings_dir
            .join(format!("{recording_id}.rrmj.json"))
    }
}

/// One entry in the Load game or Replays list.
#[derive(Debug, Clone)]
pub struct RecordingEntry {
    pub path: PathBuf,
    pub recording_id: String,
    pub label: String,
    pub detail: String,
}

pub fn list_in_progress(paths: &SavePaths) -> Result<Vec<RecordingEntry>, AppError> {
    list_by_status(paths, MatchStatus::InProgress)
}

fn list_by_status(paths: &SavePaths, status: MatchStatus) -> Result<Vec<RecordingEntry>, AppError> {
    paths.ensure_dir()?;
    let mut entries = Vec::new();

    let read_dir = match fs::read_dir(&paths.recordings_dir) {
        Ok(dir) => dir,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(entries),
        Err(err) => return Err(AppError::Terminal(err)),
    };

    for entry in read_dir {
        let entry = entry.map_err(AppError::Terminal)?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && let Some(item) = parse_recording_entry(&path, status)?
        {
            entries.push(item);
        }
    }

    entries.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(entries)
}

fn parse_recording_entry(
    path: &Path,
    status: MatchStatus,
) -> Result<Option<RecordingEntry>, AppError> {
    let text = fs::read_to_string(path).map_err(AppError::Terminal)?;
    let recording = MatchRecording::from_json(&text).map_err(AppError::Engine)?;
    if recording.match_status != status {
        return Ok(None);
    }

    let recording_id = recording
        .meta
        .recording_id
        .clone()
        .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().into_owned());

    let label = recording.meta.title.clone().unwrap_or_else(|| {
        format!(
            "Round {} · kyoku {}",
            recording.round_wind.as_str(),
            recording.kyoku
        )
    });

    let detail = format!(
        "East seat {} · honba {} · scores {:?}",
        recording.dealer, recording.honba, recording.scores
    );

    Ok(Some(RecordingEntry {
        path: path.to_path_buf(),
        recording_id,
        label,
        detail,
    }))
}

pub fn read_recording(path: &Path) -> Result<MatchRecording, AppError> {
    let text = fs::read_to_string(path).map_err(AppError::Terminal)?;
    MatchRecording::from_json(&text).map_err(AppError::Engine)
}

/// Write a recording without blocking the UI thread.
pub fn write_recording_async(path: PathBuf, recording: MatchRecording) {
    let json = match recording.to_json() {
        Ok(text) => text,
        Err(err) => {
            tracing::warn!("recording serialize failed: {err}");
            return;
        }
    };

    thread::spawn(move || {
        if let Some(parent) = path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            tracing::warn!("create recordings dir {}: {err}", parent.display());
            return;
        }
        if let Err(err) = fs::write(&path, json) {
            tracing::warn!("write recording {}: {err}", path.display());
        }
    });
}

pub fn unix_timestamp_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}
