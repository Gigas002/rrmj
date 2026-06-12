use std::fs;
use std::path::PathBuf;

use librrmj::replay::GameStatus;
use tempfile::TempDir;

use super::{
    SavePaths, ensure_recording_extension, list_finished, list_in_progress, resolve_user_path,
    write_recording,
};

fn recording_from_json(text: &str) -> librrmj::replay::GameRecording {
    librrmj::replay::GameRecording::from_json(text).expect("parse recording")
}

#[test]
fn list_filters_by_match_status() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("recordings");
    fs::create_dir_all(&dir).unwrap();

    let paths = SavePaths {
        recordings_dir: dir.clone(),
    };

    let in_progress = include_str!("../../../examples/scenarios/dealer_tsumo.json");
    fs::write(dir.join("active.json"), in_progress).unwrap();

    let mut finished = recording_from_json(in_progress);
    finished.game_status = GameStatus::Finished;
    fs::write(dir.join("done.json"), finished.to_json().unwrap()).unwrap();

    let saves = list_in_progress(&paths).unwrap();
    assert_eq!(saves.len(), 1);
    assert!(saves[0].path.ends_with("active.json"));

    let replays = list_finished(&paths).unwrap();
    assert_eq!(replays.len(), 1);
    assert!(replays[0].path.ends_with("done.json"));
    assert!(replays[0].detail.contains("hands"));
}

#[test]
fn resolve_user_path_expands_tilde() {
    let home = std::env::var_os("HOME").expect("HOME");
    let path = resolve_user_path("~/exports/save.rrmj.json");
    assert_eq!(path, PathBuf::from(home).join("exports/save.rrmj.json"));
}

#[test]
fn ensure_recording_extension_appends_suffix() {
    let path = ensure_recording_extension(PathBuf::from("/tmp/my-export"));
    assert_eq!(path, PathBuf::from("/tmp/my-export.rrmj.json"));

    let existing = ensure_recording_extension(PathBuf::from("/tmp/already.rrmj.json"));
    assert_eq!(existing, PathBuf::from("/tmp/already.rrmj.json"));
}

#[test]
fn write_recording_round_trip() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("export.rrmj.json");
    let text = include_str!("../../../examples/scenarios/dealer_tsumo.json");
    let recording = recording_from_json(text);

    write_recording(&path, &recording).unwrap();
    let read_back = fs::read_to_string(&path).unwrap();
    let parsed = recording_from_json(&read_back);
    parsed.validate().unwrap();
    assert_eq!(parsed.seed, recording.seed);
}

#[test]
fn parse_fixture_validates() {
    let text = include_str!("../../../examples/scenarios/dealer_tsumo.json");
    let parsed = recording_from_json(text);
    parsed.validate().unwrap();
    assert_eq!(parsed.game_status, GameStatus::InProgress);
}

#[test]
fn capture_promotes_finished_match_to_replay() {
    let text = include_str!("../../../examples/scenarios/match_finished.json");
    let recording = recording_from_json(text);
    assert_eq!(recording.game_status, GameStatus::Finished);

    let game = recording.restore().unwrap();
    assert!(game.is_ended());

    let setup = recording.game_setup();
    let captured = librrmj::replay::GameRecording::capture(
        &game,
        &setup,
        recording.human_seat.unwrap_or(0),
        recording.cpu_step_delay_ms.unwrap_or(300),
        recording.turn_timer_ms.unwrap_or(30_000),
        recording.response_timer_ms.unwrap_or(5_000),
        recording.meta.clone(),
    );
    assert_eq!(captured.game_status, GameStatus::Finished);
    assert_eq!(captured.meta.title, recording.meta.title);
}

#[test]
fn in_progress_fixture_restores_playable_state() {
    let text = include_str!("../../../examples/scenarios/dealer_tsumo.json");
    let recording = recording_from_json(text);
    assert_eq!(recording.game_status, GameStatus::InProgress);

    let game = recording.restore().unwrap();
    assert!(!game.is_ended());
    assert_eq!(game.events().len(), recording.events.len());
}
