use std::fs;

use librrmj::replay::MatchStatus;
use tempfile::TempDir;

use super::{SavePaths, list_by_status, list_in_progress};

fn recording_from_json(text: &str) -> librrmj::replay::MatchRecording {
    librrmj::replay::MatchRecording::from_json(text).expect("parse recording")
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
    finished.match_status = MatchStatus::Finished;
    fs::write(dir.join("done.json"), finished.to_json().unwrap()).unwrap();

    let saves = list_in_progress(&paths).unwrap();
    assert_eq!(saves.len(), 1);
    assert!(saves[0].path.ends_with("active.json"));

    let replays = list_by_status(&paths, MatchStatus::Finished).unwrap();
    assert_eq!(replays.len(), 1);
    assert!(replays[0].path.ends_with("done.json"));
}

#[test]
fn parse_fixture_validates() {
    let text = include_str!("../../../examples/scenarios/dealer_tsumo.json");
    let parsed = recording_from_json(text);
    parsed.validate().unwrap();
    assert_eq!(parsed.match_status, MatchStatus::InProgress);
}
