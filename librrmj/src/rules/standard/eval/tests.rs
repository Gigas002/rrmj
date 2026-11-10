use std::str::FromStr;

use super::*;
use crate::hand::{Concealed, Hand, Meld};
use crate::rules::{RulesProfileId, RulesRegistry, WinTimingFlags};
use crate::state::HandState;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn tiles(text: &str) -> Vec<Tile> {
    text.split_whitespace()
        .map(|t| Tile::from_str(t).unwrap_or_else(|_| panic!("tile {t}")))
        .collect()
}

/// Seat 2 (West) wins in an East round, dealer 0, after the first go-around.
fn eval_with(
    concealed: &str,
    melds: Vec<Meld>,
    win_type: WinType,
    win: &str,
) -> Option<Evaluation> {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(5));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config.clone());
    state.set_hand(
        2,
        Hand::new(Concealed::from_tiles(tiles(concealed)), melds).unwrap(),
    );
    state.is_dealer_first_turn = false;
    state.live_draws = [1; 4];
    state.calls_made = true;
    let ctx = WinContext::new(
        &state,
        2,
        win_type,
        Tile::from_str(win).unwrap(),
        WinTimingFlags::default(),
    );
    evaluate(&ctx, &config, 0)
}

fn ron(concealed: &str, win: &str) -> Option<Evaluation> {
    eval_with(concealed, vec![], WinType::Ron { from: 0 }, win)
}

fn closed_kan(tile: &str) -> Meld {
    let t = Tile::from_str(tile).unwrap();
    Meld::closed_kan([t; 4]).unwrap()
}

fn pon(tile: &str) -> Meld {
    let t = Tile::from_str(tile).unwrap();
    Meld::pon([t; 3], t).unwrap()
}

#[test]
fn a_dragon_pair_is_not_yakuhai() {
    assert_eq!(ron("1m 2m 3m 4p 5p 6p 7s 8s 9s 2m 3m wd wd", "4m"), None);
}

#[test]
fn each_value_triplet_counts_and_double_wind_counts_twice() {
    let eval = ron("wd wd wd rd rd rd 2m 3m 4m 6p 7p 8p 9s", "9s").unwrap();
    assert_eq!(eval.yaku, vec![Yaku::Yakuhai, Yaku::Yakuhai]);

    // Dealer (East seat, East round) with an East triplet: two yakuhai.
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(6));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config.clone());
    state.set_hand(
        0,
        Hand::new(
            Concealed::from_tiles(tiles("E E E 2m 3m 4m 6p 7p 8p 3s 4s 5s 9s")),
            vec![],
        )
        .unwrap(),
    );
    state.is_dealer_first_turn = false;
    state.calls_made = true;
    let ctx = WinContext::new(
        &state,
        0,
        WinType::Ron { from: 1 },
        Tile::from_str("9s").unwrap(),
        WinTimingFlags::default(),
    );
    let result = profile.score_win(&ctx, &config);
    assert_eq!(result.yaku, vec![Yaku::Yakuhai, Yaku::Yakuhai]);
}

#[test]
fn the_highest_scoring_reading_wins() {
    // 111222333m: three triplets (sanankou) or three 123m runs (pinfu,
    // iipeikou and junchan with the 789s and 99s). The runs score more.
    let eval = ron("1m 1m 1m 2m 2m 2m 3m 3m 3m 7s 8s 9s 9s", "9s").unwrap();
    assert!(eval.yaku.contains(&Yaku::Junchan), "{:?}", eval.yaku);
    assert!(eval.yaku.contains(&Yaku::Iipeikou), "{:?}", eval.yaku);
    assert!(!eval.yaku.contains(&Yaku::Sanankou));
}

#[test]
fn ron_on_a_shanpon_opens_that_triplet() {
    // 222m 444p concealed, 66s/88s shanpon: ron on 8s leaves two concealed
    // triplets (no sanankou); tsumo on 8s makes three.
    let hand = "2m 2m 2m 4p 4p 4p 6s 6s 8s 8s 3m 4m 5m";
    let ron_eval = ron(hand, "8s").unwrap();
    assert!(
        !ron_eval.yaku.contains(&Yaku::Sanankou),
        "{:?}",
        ron_eval.yaku
    );
    // 20 + 4 + 4 + open 8s triplet 2 + closed ron 10 = 40.
    assert_eq!(ron_eval.fu, 40);

    let mut tsumo_hand = tiles(hand);
    tsumo_hand.push(Tile::sou(8));
    let text: Vec<String> = tsumo_hand.iter().map(|t| t.to_string()).collect();
    let tsumo = eval_with(&text.join(" "), vec![], WinType::Tsumo, "8s").unwrap();
    assert!(tsumo.yaku.contains(&Yaku::Sanankou), "{:?}", tsumo.yaku);
}

#[test]
fn closed_kans_keep_the_hand_closed() {
    let eval = eval_with(
        "2m 3m 4m 6p 7p 8p 3s 4s 5s 6s 6s",
        vec![closed_kan("5p")],
        WinType::Tsumo,
        "6s",
    )
    .unwrap();
    assert!(eval.yaku.contains(&Yaku::MenzenTsumo), "{:?}", eval.yaku);
    assert!(eval.yaku.contains(&Yaku::Tanyao));
}

#[test]
fn added_yaku_are_detected() {
    let sanshoku_doukou = ron("3m 3m 3m 3p 3p 3p 3s 3s 3s 5m 6m 7m 9p", "9p").unwrap();
    assert!(
        sanshoku_doukou.yaku.contains(&Yaku::SanshokuDoukou),
        "{:?}",
        sanshoku_doukou.yaku
    );

    let shousangen = ron("wd wd wd gd gd gd rd rd 2m 3m 4m 6p 7p", "8p").unwrap();
    assert!(
        shousangen.yaku.contains(&Yaku::Shousangen),
        "{:?}",
        shousangen.yaku
    );
    assert_eq!(
        shousangen
            .yaku
            .iter()
            .filter(|y| **y == Yaku::Yakuhai)
            .count(),
        2
    );

    let honroutou = eval_with(
        "1m 1m 1m 9p 9p 9p E E E N N",
        vec![pon("9s")],
        WinType::Ron { from: 0 },
        "N",
    )
    .unwrap();
    assert!(
        honroutou.yaku.contains(&Yaku::Honroutou),
        "{:?}",
        honroutou.yaku
    );
    assert!(honroutou.yaku.contains(&Yaku::Toitoi));

    let open_kan = Meld::open_kan([Tile::sou(7); 4], Tile::sou(7)).unwrap();
    let sankantsu = eval_with(
        "2m 3m 4m 9p",
        vec![closed_kan("5p"), open_kan, closed_kan("E")],
        WinType::Ron { from: 0 },
        "9p",
    )
    .unwrap();
    assert!(
        sankantsu.yaku.contains(&Yaku::Sankantsu),
        "{:?}",
        sankantsu.yaku
    );
}

#[test]
fn added_yakuman_are_detected() {
    let tsuuiisou = eval_with(
        "E E E S S S wd wd wd rd rd",
        vec![pon("N")],
        WinType::Ron { from: 0 },
        "rd",
    )
    .unwrap();
    assert!(
        tsuuiisou.yaku.contains(&Yaku::Tsuuiisou),
        "{:?}",
        tsuuiisou.yaku
    );

    let seven_pair_honors = ron("E E S S W W N N wd wd gd gd rd", "rd").unwrap();
    assert_eq!(seven_pair_honors.yaku, vec![Yaku::Tsuuiisou]);

    let chinroutou = eval_with(
        "1m 1m 1m 9m 9m 9m 1p 1p 1p 9s 9s",
        vec![pon("1s")],
        WinType::Ron { from: 0 },
        "9s",
    )
    .unwrap();
    assert_eq!(chinroutou.yaku, vec![Yaku::Chinroutou]);
}

#[test]
fn four_of_a_kind_is_not_two_pairs() {
    // 2m x4 + five pairs is not chiitoitsu (and no other reading wins).
    assert_eq!(ron("2m 2m 2m 2m 4p 4p 6p 6p 8s 8s 3s 3s 7m", "7m"), None);
}

#[test]
fn fu_round_up_to_ten() {
    // Closed ron, kanchan: 20 + 10 + 2 = 32 -> 40.
    let eval = ron("2m 4m 3p 4p 5p 6s 7s 8s 5p 6p 7p 8m 8m", "3m").unwrap();
    assert_eq!(eval.yaku, vec![Yaku::Tanyao]);
    assert_eq!(eval.fu, 40);
}
