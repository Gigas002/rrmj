use super::build_from_context;
use crate::hand::{Concealed, Hand};
use crate::rules::RulesConfig;
use crate::rules::profile::{WinContext, WinTimingFlags};
use crate::scoring::WinType;
use crate::state::HandState;
use crate::tile::Tile;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn state_with_hand(hand: Hand) -> HandState {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(7));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_hand(0, hand);
    state
}

#[test]
fn pinfu_path_shows_sequences_and_wait() {
    let concealed = vec![
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
    ];
    let hand = Hand::new(Concealed::from_tiles(concealed), vec![]).unwrap();
    let state = state_with_hand(hand);
    let ctx = WinContext::new(
        &state,
        0,
        WinType::Tsumo,
        Tile::pin(2),
        WinTimingFlags::default(),
    );
    let decomp = build_from_context(&ctx, 0, None);
    assert!(!decomp.groups.is_empty());
    assert_eq!(decomp.missing, vec![Tile::pin(2)]);
    let lines = decomp.format_lines();
    assert!(lines.iter().any(|line| line.contains("2m3m4m")));
    assert!(lines.iter().any(|line| line.starts_with("Need +2p")));
}

#[test]
fn one_shanten_includes_suggested_discard() {
    let concealed = vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(5),
        Tile::pin(6),
        Tile::pin(7),
        Tile::sou(3),
        Tile::sou(4),
        Tile::sou(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(9),
        Tile::pin(2),
    ];
    let hand = Hand::new(Concealed::from_tiles(concealed), vec![]).unwrap();
    let state = state_with_hand(hand);
    let ctx = WinContext::new(
        &state,
        0,
        WinType::Tsumo,
        Tile::pin(2),
        WinTimingFlags::default(),
    );
    let decomp = build_from_context(&ctx, 1, Some(Tile::sou(9)));
    assert_eq!(decomp.suggested_discard, Some(Tile::sou(9)));
    assert!(
        decomp
            .format_lines()
            .first()
            .is_some_and(|line| line.starts_with("Discard 9s"))
    );
}
