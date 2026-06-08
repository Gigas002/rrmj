use crate::action::Action;
use crate::game::Match;
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
        Tile::sou(9),
        Tile::sou(9),
        Tile::sou(9),
        Tile::pin(2),
        Tile::pin(2),
    ]
}

fn play_tsumo_hand(seed: u64) -> Match {
    let mut game = Match::new(RulesConfig::standard(), seed).unwrap();
    let winner = game.dealer();
    game.hand_mut()
        .set_concealed(winner, winning_tanyao_tiles());
    game.hand_mut().last_draw = Some(Tile::pin(2));
    game.apply_action(winner, Action::Tsumo).unwrap();
    game
}

#[test]
fn replay_apply_all_matches_live_play() {
    let live = play_tsumo_hand(42);
    let replay = Replay::from_match(&live);
    let replayed = replay.apply_all().unwrap();
    assert_eq!(live.snapshot(), replayed.snapshot());
}

#[test]
fn replay_snapshots_end_at_live_state() {
    let live = play_tsumo_hand(43);
    let replay = Replay::from_match(&live);
    let snapshots = replay.snapshots().unwrap();
    assert_eq!(*snapshots.last().unwrap(), live.snapshot());
}

#[test]
#[cfg(feature = "serde")]
fn replay_serde_round_trip() {
    let live = play_tsumo_hand(7);
    let replay = Replay::from_match(&live);
    let json = serde_json::to_string(&replay).unwrap();
    let decoded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(replay, decoded);
}
