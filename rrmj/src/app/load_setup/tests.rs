use librrmj::agent::PlayerSlot;
use librrmj::replay::GameRecording;

use super::LoadGameSetup;
use crate::save::RecordingEntry;

#[test]
fn rejects_finished_recording_for_load() {
    let text = include_str!("../../../../examples/scenarios/match_finished.json");
    let recording = GameRecording::from_json(text).unwrap();
    assert_ne!(
        recording.game_status,
        librrmj::replay::GameStatus::InProgress
    );
}

#[test]
fn remaps_agents_when_study_seat_chosen() {
    let text = include_str!("../../../../examples/scenarios/dealer_tsumo.json");
    let recording = GameRecording::from_json(text).unwrap();
    let entry = RecordingEntry {
        path: "test.json".into(),
        recording_id: "test".into(),
        label: "test".into(),
        detail: String::new(),
    };
    let mut load = LoadGameSetup::new(entry, recording, 0, 300, 30_000, 5_000);
    load.selected_seat = 1;

    let setup = load.game_setup_for_load();
    assert_eq!(setup.slots[1], PlayerSlot::Human);
    assert_eq!(setup.slots[0], PlayerSlot::Cpu);
}
