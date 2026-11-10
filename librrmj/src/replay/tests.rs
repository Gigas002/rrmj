use crate::action::Action;
use crate::game::Game;
use crate::replay::Replay;
use crate::rules::RulesConfig;
use crate::tile::Tile;

fn winning_tanyao_tiles() -> Vec<Tile> {
    vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(3),
        Tile::sou(4),
        Tile::sou(5),
        Tile::pin(2),
        Tile::pin(2),
    ]
}

fn force_tsumo_win(game: &mut Game, seat: usize) {
    let hand = game.hand_mut();
    hand.set_concealed(seat, winning_tanyao_tiles());
    hand.last_draw = Some(Tile::pin(2));
}

fn play_tsumo_hand(seed: u64) -> Game {
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    let winner = game.dealer();
    force_tsumo_win(&mut game, winner);
    game.apply_action(winner, Action::Tsumo).unwrap();
    game
}

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

/// A ron leaves the winning tile in the discarder's river: the ended hand (what
/// a finished recording stores, and what replay rebuilds) keeps all 136 tiles.
#[cfg(feature = "serde")]
#[test]
fn ron_keeps_the_winning_tile_in_the_river() {
    use crate::replay::GameRecording;
    use crate::state::{HandPhase, HandState};

    let text = include_str!("../../../examples/scenarios/ron_on_discard.json");
    let recording = GameRecording::from_json(text).unwrap();
    let mut hand =
        HandState::from_snapshot(recording.hand.clone(), recording.rules_config.clone()).unwrap();
    let reaction = recording.hand.reaction.clone().expect("ron reaction");
    let river_before = hand.discards(reaction.discarder).len();

    for seat in 0..4 {
        if hand.phase() != HandPhase::Reaction {
            break;
        }
        let legal = hand.legal_actions_for(seat);
        if legal.contains(&Action::Ron) {
            hand.apply(seat, Action::Ron).unwrap();
        } else if legal.contains(&Action::Pass) {
            hand.apply(seat, Action::Pass).unwrap();
        }
    }

    assert_eq!(hand.phase(), HandPhase::Ended);
    assert_eq!(hand.discards(reaction.discarder).len(), river_before);
    assert_eq!(
        hand.discards(reaction.discarder).last(),
        Some(&reaction.tile)
    );
    hand.validate_tile_conservation().unwrap();
}

/// Whole multi-hand matches recorded and replayed: every hand boundary and
/// every live step can be sought, and the boards (hands, melds, rivers,
/// scores) match the live match at each point. Seeds run until pon, chi,
/// closed kan and kakan have all been replayed.
#[cfg(feature = "serde")]
#[test]
fn full_match_replays_hand_after_hand() {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::ai::{AiConfig, CpuAgent, Difficulty, GameSetup};
    use crate::event::Event;
    use crate::game::GameLength;
    use crate::hand::{KanForm, MeldKind};
    use crate::replay::{GameRecording, RecordingMeta, RecordingPlayer};

    fn board(game: &Game) -> String {
        let hand = game.hand();
        let seats: Vec<String> = (0..4)
            .map(|seat| {
                format!(
                    "{:?}|{:?}|{:?}",
                    hand.hand(seat).concealed().tiles(),
                    hand.hand(seat).melds(),
                    hand.discards(seat)
                )
            })
            .collect();
        format!("{:?} {:?}", seats, game.scores())
    }

    let mut config = RulesConfig::standard();
    config.game_length = GameLength::EastOnly;
    let wanted = ["chi", "closed kan", "kakan", "pon"];
    let mut seen = BTreeSet::new();
    for seed in 4242..4300 {
        let mut live = Game::new(config.clone(), seed).unwrap();
        let mut agents: [CpuAgent; 4] = std::array::from_fn(|seat| {
            CpuAgent::new(AiConfig::new(Difficulty::Easy, seed), seat, seed)
        });
        let mut boards = BTreeMap::new();
        while let Some(_step) = live.step(&mut agents).unwrap() {
            boards.insert(live.events().len(), board(&live));
        }
        assert!(live.is_ended());
        for event in live.events() {
            match event {
                Event::Called {
                    meld: MeldKind::Chi,
                    ..
                } => seen.insert("chi"),
                Event::Called {
                    meld: MeldKind::Pon,
                    ..
                } => seen.insert("pon"),
                Event::Called {
                    meld: MeldKind::Kan(KanForm::Closed),
                    ..
                } => seen.insert("closed kan"),
                Event::KakanDeclared { .. } => seen.insert("kakan"),
                _ => false,
            };
        }

        let recording = GameRecording::capture(
            &live,
            &GameSetup::all_medium(seed),
            0,
            0,
            0,
            0,
            RecordingMeta::default(),
        );
        let mut player = RecordingPlayer::new(recording).unwrap();
        for hand in 0..player.hand_boundaries().len() {
            player.seek_hand(hand).unwrap();
        }
        for (&len, want) in &boards {
            player.seek(Some(len - 1)).unwrap();
            let got = board(player.game());
            assert_eq!(&got, want, "seed {seed}, event {}", len - 1);
        }
        assert!(player.at_end());
        assert_eq!(player.game().scores(), live.scores());
        if wanted.iter().all(|kind| seen.contains(kind)) {
            return;
        }
    }
    panic!("seeds never covered all of {wanted:?}; saw {seen:?}");
}
