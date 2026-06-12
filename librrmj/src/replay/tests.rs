use crate::replay::Replay;
use crate::test_util::fixtures::play_tsumo_hand;

#[test]
fn replay_apply_all_matches_live_play() {
    let live = play_tsumo_hand(42);
    let replay = Replay::from_game(&live);
    let replayed = replay.apply_all().unwrap();
    assert_eq!(live.snapshot(), replayed.snapshot());
}

#[test]
fn replay_snapshots_end_at_live_state() {
    let live = play_tsumo_hand(43);
    let replay = Replay::from_game(&live);
    let snapshots = replay.snapshots().unwrap();
    assert_eq!(*snapshots.last().unwrap(), live.snapshot());
}

#[test]
#[cfg(feature = "serde")]
fn replay_serde_round_trip() {
    let live = play_tsumo_hand(7);
    let replay = Replay::from_game(&live);
    let json = serde_json::to_string(&replay).unwrap();
    let decoded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(replay, decoded);
}

#[cfg(feature = "serde")]
mod recording {
    use super::*;
    use crate::ai::GameSetup;
    use crate::replay::{GameRecording, RecordingMeta, RecordingPlayer};

    #[test]
    fn recording_capture_restore_round_trip() {
        let live = play_tsumo_hand(101);
        let setup = GameSetup::all_medium(live.seed());
        let recording = GameRecording::capture(
            &live,
            &setup,
            0,
            300,
            30_000,
            5_000,
            RecordingMeta::default(),
        );
        let restored = recording.restore().unwrap();

        assert_eq!(live.snapshot(), restored.snapshot());
        assert_eq!(live.events(), restored.events());
        assert_eq!(
            live.pending_legal_actions(),
            restored.pending_legal_actions()
        );
    }

    #[test]
    fn recording_json_round_trip() {
        let live = play_tsumo_hand(102);
        let setup = GameSetup::all_easy(live.seed());
        let recording = GameRecording::capture(
            &live,
            &setup,
            1,
            300,
            30_000,
            5_000,
            RecordingMeta::default(),
        );
        let json = recording.to_json().unwrap();
        let decoded = GameRecording::from_json(&json).unwrap();
        let restored = decoded.restore().unwrap();
        assert_eq!(live.snapshot(), restored.snapshot());
    }

    #[test]
    fn recording_apply_until_matches_live_at_index() {
        let live = play_tsumo_hand(103);
        let setup = GameSetup::all_medium(live.seed());
        let recording = GameRecording::capture(
            &live,
            &setup,
            0,
            300,
            30_000,
            5_000,
            RecordingMeta::default(),
        );
        let index = recording.event_index;
        let at_checkpoint = recording.apply_until(index).unwrap();
        assert_eq!(live.snapshot(), at_checkpoint.snapshot());
    }

    #[test]
    fn recording_tile_conservation() {
        let live = play_tsumo_hand(104);
        let setup = GameSetup::all_medium(live.seed());
        let recording = GameRecording::capture(
            &live,
            &setup,
            0,
            300,
            30_000,
            5_000,
            RecordingMeta::default(),
        );
        recording.validate().unwrap();
        let restored = recording.restore().unwrap();
        restored.hand().validate_tile_conservation().unwrap();
    }

    #[test]
    fn recording_player_steps_to_restore_snapshot() {
        let live = play_tsumo_hand(106);
        let setup = GameSetup::all_medium(live.seed());
        let recording = GameRecording::capture(
            &live,
            &setup,
            0,
            300,
            30_000,
            5_000,
            RecordingMeta::default(),
        );
        let mut player = RecordingPlayer::new(recording).unwrap();
        while player.step_forward().unwrap() {}
        assert_eq!(player.game().snapshot(), live.snapshot());
    }

    #[test]
    fn recording_player_seek_matches_apply_until() {
        let live = play_tsumo_hand(107);
        let setup = GameSetup::all_medium(live.seed());
        let recording = GameRecording::capture(
            &live,
            &setup,
            0,
            300,
            30_000,
            5_000,
            RecordingMeta::default(),
        );
        let index = recording.event_index;
        let mut player = RecordingPlayer::new(recording.clone()).unwrap();
        player.play_to_index(index).unwrap();
        let via_apply = recording.apply_until(index).unwrap();
        assert_eq!(player.game().snapshot(), via_apply.snapshot());
    }

    #[test]
    fn recording_player_step_back_reaches_start() {
        let live = play_tsumo_hand(108);
        let setup = GameSetup::all_medium(live.seed());
        let recording = GameRecording::capture(
            &live,
            &setup,
            0,
            300,
            30_000,
            5_000,
            RecordingMeta::default(),
        );
        let mut player = RecordingPlayer::new(recording).unwrap();
        assert!(player.step_forward().unwrap());
        assert!(!player.at_start());
        while player.step_back().unwrap() {}
        assert!(player.at_start());
    }

    #[test]
    fn replay_still_works_after_refactor() {
        let live = play_tsumo_hand(105);
        let replay = Replay::from_game(&live);
        let replayed = replay.apply_all().unwrap();
        assert_eq!(live.snapshot(), replayed.snapshot());
    }
}
