use std::str::FromStr;

use crate::hand::{Concealed, Hand, Meld};
use crate::rules::standard::fu::calculate_fu;
use crate::rules::{RulesConfig, WinContext, WinTimingFlags};
use crate::scoring::{WinType, Yaku};
use crate::state::HandState;
use crate::tile::Tile;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[allow(clippy::too_many_arguments)]
fn fu_for(
    concealed: &[&str],
    melds: &[(&str, [&str; 3], &str)],
    winner: usize,
    dealer: usize,
    win_type: WinType,
    win_tile: &str,
    yaku: &[Yaku],
    config: &RulesConfig,
) -> u8 {
    let melds: Vec<Meld> = melds
        .iter()
        .map(|(kind, tiles, called)| {
            let tiles: [Tile; 3] = [
                Tile::from_str(tiles[0]).unwrap(),
                Tile::from_str(tiles[1]).unwrap(),
                Tile::from_str(tiles[2]).unwrap(),
            ];
            let called = Tile::from_str(called).unwrap();
            match *kind {
                "pon" => Meld::pon(tiles, called).unwrap(),
                _ => panic!("unknown meld"),
            }
        })
        .collect();
    let concealed: Vec<Tile> = concealed
        .iter()
        .map(|s| Tile::from_str(s).unwrap())
        .collect();
    let hand = Hand::new(Concealed::from_tiles(concealed), melds).unwrap();
    let mut wall = Wall::new(config, StdRng::seed_from_u64(99));
    let deal = wall.deal(dealer).unwrap();
    let mut state = HandState::from_deal(wall, deal, config.clone());
    state.set_hand(winner, hand);
    let win_tile = Tile::from_str(win_tile).unwrap();
    let ctx = WinContext::new(
        &state,
        winner,
        win_type,
        win_tile,
        WinTimingFlags::default(),
    );
    calculate_fu(&ctx, yaku, config)
}

#[test]
fn pinfu_ron_is_30() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &[
                "1m", "2m", "3m", "5m", "6m", "7m", "3p", "4p", "5p", "8p", "8p", "2s", "3s",
            ],
            &[],
            0,
            0,
            WinType::Ron { from: 2 },
            "4s",
            &[Yaku::Pinfu],
            &config,
        ),
        30
    );
}

#[test]
fn pinfu_tsumo_is_20() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &[
                "1m", "2m", "3m", "5m", "6m", "7m", "3p", "4p", "5p", "8p", "8p", "2s", "3s", "4s",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "4s",
            &[Yaku::Pinfu, Yaku::MenzenTsumo],
            &config,
        ),
        20
    );
}

#[test]
fn chiitoitsu_is_25() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &[
                "2m", "2m", "3m", "3m", "4m", "4m", "5p", "5p", "6p", "6p", "7s", "7s", "8s", "8s",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "8s",
            &[Yaku::Chiitoitsu, Yaku::MenzenTsumo],
            &config,
        ),
        25
    );
}

#[test]
fn open_zero_fu_ron_is_30() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &["2m", "3m", "4m", "3p", "4p", "5p", "6p", "7p", "8p", "2p"],
            &[("pon", ["5s", "5s", "5s"], "5s")],
            1,
            0,
            WinType::Ron { from: 0 },
            "2p",
            &[Yaku::Tanyao],
            &config,
        ),
        30
    );
}

#[test]
fn closed_ron_adds_10() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "3s", "4s", "5s", "6p", "7p", "8p", "2m",
            ],
            &[],
            0,
            0,
            WinType::Ron { from: 2 },
            "2m",
            &[Yaku::Tanyao],
            &config,
        ),
        32
    );
}

#[test]
fn yakuhai_ron_triplet_adds_meld_and_wait_fu() {
    let config = RulesConfig::standard();
    // Ron on third E completes closed triplet: base 20 + triplet 8 + wait 2 + closed ron 10 = 40
    assert_eq!(
        fu_for(
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "6s", "7s", "8s", "3p", "3p", "E", "E",
            ],
            &[],
            0,
            0,
            WinType::Ron { from: 2 },
            "E",
            &[Yaku::Yakuhai],
            &config,
        ),
        40
    );
}

#[test]
fn open_pon_simple_adds_2() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6p", "7p", "8p", "2p", "2p"
            ],
            &[("pon", ["5s", "5s", "5s"], "5s")],
            1,
            0,
            WinType::Tsumo,
            "2p",
            &[Yaku::Tanyao],
            &config,
        ),
        30
    );
}

#[test]
fn concealed_triplet_simple_adds_4() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p",
            ],
            &[],
            1,
            0,
            WinType::Ron { from: 0 },
            "2p",
            &[Yaku::Tanyao],
            &config,
        ),
        36
    );
}

#[test]
fn kiriage_rounds_up() {
    let mut config = RulesConfig::standard();
    config.kiriage = true;
    assert_eq!(
        fu_for(
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p",
            ],
            &[],
            1,
            0,
            WinType::Ron { from: 0 },
            "2p",
            &[Yaku::Tanyao],
            &config,
        ),
        40
    );
}

#[test]
fn chiitoitsu_kiriage_rounds_to_30() {
    let mut config = RulesConfig::standard();
    config.kiriage = true;
    assert_eq!(
        fu_for(
            &[
                "2m", "2m", "3m", "3m", "4m", "4m", "5p", "5p", "6p", "6p", "7s", "7s", "8s", "8s",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "8s",
            &[Yaku::Chiitoitsu, Yaku::MenzenTsumo],
            &config,
        ),
        30
    );
}

#[test]
fn tsumo_adds_2_except_pinfu() {
    let config = RulesConfig::standard();
    assert_eq!(
        fu_for(
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ],
            &[],
            1,
            0,
            WinType::Tsumo,
            "2p",
            &[Yaku::Tanyao, Yaku::MenzenTsumo],
            &config,
        ),
        28
    );
}
