use crate::hand::{Concealed, Hand};
use crate::rules::standard::score::score_hand;
use crate::rules::{RulesConfig, WinContext, WinTimingFlags};
use crate::scoring::limits::base_points;
use crate::scoring::{WinType, Yaku};
use crate::state::HandState;
use crate::tile::Tile;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn score_for(
    dealer: usize,
    winner: usize,
    win_type: WinType,
    han: u8,
    fu: u8,
    honba: u8,
    riichi_sticks: u8,
) -> [i32; 4] {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(7));
    let deal = wall.deal(dealer).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    let hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::pin(3),
            Tile::pin(4),
            Tile::pin(5),
            Tile::sou(6),
            Tile::sou(7),
            Tile::sou(8),
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(5),
            Tile::pin(2),
            Tile::pin(2),
        ]),
        vec![],
    )
    .unwrap();
    state.set_hand(winner, hand);
    state.set_honba_for_test(honba);
    state.set_table_riichi_sticks_for_test(riichi_sticks);
    let win_tile = Tile::pin(2);
    let ctx = WinContext::new(
        &state,
        winner,
        win_type,
        win_tile,
        WinTimingFlags::default(),
    );
    score_hand(
        &ctx,
        &[Yaku::Tanyao],
        han,
        fu,
        0,
        0,
        0,
        &RulesConfig::standard(),
    )
    .deltas
}

#[test]
fn base_points_mangan_thresholds() {
    assert_eq!(base_points(5, 30), 2_000);
    assert_eq!(base_points(4, 40), 2_000);
    assert_eq!(base_points(3, 70), 2_000);
    assert_eq!(base_points(4, 39), 39 * 64);
    assert_eq!(base_points(3, 69), 69 * 32);
}

#[test]
fn base_points_limit_bands() {
    assert_eq!(base_points(6, 30), 3_000);
    assert_eq!(base_points(7, 30), 3_000);
    assert_eq!(base_points(8, 30), 4_000);
    assert_eq!(base_points(10, 30), 4_000);
    assert_eq!(base_points(11, 30), 6_000);
    assert_eq!(base_points(12, 30), 6_000);
    assert_eq!(base_points(13, 30), 8_000);
    assert_eq!(base_points(26, 30), 8_000);
}

#[test]
fn ko_ron_payments() {
    let deltas = score_for(0, 1, WinType::Ron { from: 2 }, 2, 30, 0, 0);
    assert_eq!(deltas[1], 2_000);
    assert_eq!(deltas[2], -2_000);

    let deltas = score_for(0, 1, WinType::Ron { from: 2 }, 1, 30, 0, 0);
    assert_eq!(deltas[1], 1_000);
    assert_eq!(deltas[2], -1_000);
}

#[test]
fn oya_ron_payments() {
    let deltas = score_for(0, 0, WinType::Ron { from: 2 }, 2, 30, 0, 0);
    assert_eq!(deltas[0], 2_900);
    assert_eq!(deltas[2], -2_900);

    let deltas = score_for(0, 0, WinType::Ron { from: 2 }, 1, 30, 0, 0);
    assert_eq!(deltas[0], 1_500);
    assert_eq!(deltas[2], -1_500);
}

#[test]
fn ko_tsumo_payments() {
    let deltas = score_for(0, 1, WinType::Tsumo, 2, 30, 0, 0);
    assert_eq!(deltas[1], 2_000);
    assert_eq!(deltas[0], -1_000);
    assert_eq!(deltas[2], -500);
    assert_eq!(deltas[3], -500);

    let deltas = score_for(0, 1, WinType::Tsumo, 3, 30, 0, 0);
    assert_eq!(deltas[1], 4_000);
    assert_eq!(deltas[0], -2_000);
    assert_eq!(deltas[2], -1_000);
    assert_eq!(deltas[3], -1_000);
}

#[test]
fn oya_tsumo_payments() {
    let deltas = score_for(0, 0, WinType::Tsumo, 2, 30, 0, 0);
    assert_eq!(deltas[0], 3_000);
    assert_eq!(deltas[1], -1_000);
    assert_eq!(deltas[2], -1_000);
    assert_eq!(deltas[3], -1_000);

    let deltas = score_for(0, 0, WinType::Tsumo, 3, 30, 0, 0);
    assert_eq!(deltas[0], 6_000);
    assert_eq!(deltas[1], -2_000);
    assert_eq!(deltas[2], -2_000);
    assert_eq!(deltas[3], -2_000);
}

#[test]
fn honba_and_riichi_sticks() {
    let ron = score_for(0, 1, WinType::Ron { from: 2 }, 2, 30, 2, 0);
    assert_eq!(ron[1], 2_600);
    assert_eq!(ron[2], -2_600);

    let tsumo = score_for(0, 1, WinType::Tsumo, 2, 30, 2, 0);
    assert_eq!(tsumo[1], 2_600);
    assert_eq!(tsumo[0], -1_200);
    assert_eq!(tsumo[2], -700);
    assert_eq!(tsumo[3], -700);

    let sticks = score_for(0, 1, WinType::Ron { from: 2 }, 2, 30, 0, 2);
    assert_eq!(sticks[1], 4_000);
    assert_eq!(sticks[2], -4_000);
}

#[test]
fn mangan_ron_payments() {
    let ko = score_for(0, 1, WinType::Ron { from: 2 }, 5, 30, 0, 0);
    assert_eq!(ko[1], 8_000);
    assert_eq!(ko[2], -8_000);

    let oya = score_for(0, 0, WinType::Ron { from: 2 }, 5, 30, 0, 0);
    assert_eq!(oya[0], 12_000);
    assert_eq!(oya[2], -12_000);
}

#[test]
fn limit_hand_payments() {
    let haneman = score_for(0, 1, WinType::Ron { from: 2 }, 6, 30, 0, 0);
    assert_eq!(haneman[1], 12_000);

    let baiman = score_for(0, 1, WinType::Ron { from: 2 }, 8, 30, 0, 0);
    assert_eq!(baiman[1], 16_000);

    let sanbaiman = score_for(0, 1, WinType::Ron { from: 2 }, 11, 30, 0, 0);
    assert_eq!(sanbaiman[1], 24_000);

    let yakuman = score_for(0, 1, WinType::Ron { from: 2 }, 13, 30, 0, 0);
    assert_eq!(yakuman[1], 32_000);
}
