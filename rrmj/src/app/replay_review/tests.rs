use librrmj::replay::{GameRecording, RecordingPlayer};

use super::ReplayReview;
use crate::save::RecordingEntry;

#[test]
fn replay_player_matches_apply_until() {
    let text = include_str!("../../../../examples/scenarios/dealer_tsumo.json");
    let recording = GameRecording::from_json(text).unwrap();
    let entry = RecordingEntry {
        path: "dealer_tsumo.json".into(),
        recording_id: "dealer_tsumo".into(),
        label: "dealer tsumo".into(),
        detail: String::new(),
    };
    let player = RecordingPlayer::new(recording.clone()).unwrap();
    let mut review = ReplayReview::new(entry, player, 0);
    review.step_forward().unwrap();
    let at_index = recording.apply_until(0).unwrap();
    assert_eq!(review.player.game().snapshot(), at_index.snapshot());
    review.set_view_seat(2);
    assert_eq!(review.view_seat, 2);
}
