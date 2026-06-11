use std::str::FromStr;

use crate::hand::{Concealed, Hand, Meld};
use crate::rules::standard::cheatsheet;
use crate::rules::standard::win_combinations::WinCase;
use crate::rules::standard::yaku::total_han;
use crate::rules::{RulesConfig, RulesProfileId, RulesRegistry, WinContext, WinTimingFlags};
use crate::scoring::{WinType, Yaku};
use crate::state::HandState;
use crate::tile::Tile;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

type MeldSpec<'a> = (&'a str, [&'a str; 3], &'a str);

/// Win fixtures for Phase 11.4 baseline yaku (`cheatsheet::implemented_rows`).
pub fn baseline_cases() -> Vec<WinCase> {
    vec![
        case(
            "menzen_tsumo_tanyao",
            "menzen_tsumo",
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ],
            &[],
            1,
            0,
            WinType::Tsumo,
            "2p",
            false,
            false,
            false,
            &[Yaku::MenzenTsumo, Yaku::Tanyao],
            &[Yaku::Pinfu, Yaku::Yakuhai, Yaku::Chiitoitsu],
            2,
            28,
        ),
        case(
            "tanyao_ron",
            "tanyao",
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p",
            ],
            &[],
            1,
            0,
            WinType::Ron { from: 0 },
            "2p",
            false,
            false,
            false,
            &[Yaku::Tanyao],
            &[Yaku::MenzenTsumo, Yaku::Pinfu],
            1,
            36,
        ),
        case(
            "tanyao_open_ron",
            "tanyao",
            &["2m", "3m", "4m", "3p", "4p", "5p", "6p", "7p", "8p", "2p"],
            &[("pon", ["5s", "5s", "5s"], "5s")],
            1,
            0,
            WinType::Ron { from: 0 },
            "2p",
            false,
            false,
            false,
            &[Yaku::Tanyao],
            &[Yaku::MenzenTsumo, Yaku::Pinfu, Yaku::Chiitoitsu],
            1,
            30,
        ),
        case(
            "pinfu_menzen_tsumo",
            "pinfu",
            &[
                "1m", "2m", "3m", "5m", "6m", "7m", "3p", "4p", "5p", "8p", "8p", "2s", "3s", "4s",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "4s",
            false,
            false,
            false,
            &[Yaku::Pinfu, Yaku::MenzenTsumo],
            &[Yaku::Tanyao, Yaku::Yakuhai],
            2,
            20,
        ),
        case(
            "pinfu_ron",
            "pinfu",
            &[
                "1m", "2m", "3m", "5m", "6m", "7m", "3p", "4p", "5p", "8p", "8p", "2s", "3s",
            ],
            &[],
            0,
            0,
            WinType::Ron { from: 2 },
            "4s",
            false,
            false,
            false,
            &[Yaku::Pinfu],
            &[Yaku::Tanyao, Yaku::MenzenTsumo],
            1,
            30,
        ),
        case(
            "yakuhai_round_wind_ron",
            "yakuhai",
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "6s", "7s", "8s", "3p", "3p", "E", "E",
            ],
            &[],
            0,
            0,
            WinType::Ron { from: 2 },
            "E",
            false,
            false,
            false,
            &[Yaku::Yakuhai],
            &[Yaku::Tanyao],
            1,
            40,
        ),
        case(
            "yakuhai_seat_wind_ron",
            "yakuhai",
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "6s", "7s", "8s", "3p", "3p", "S", "S",
            ],
            &[],
            1,
            0,
            WinType::Ron { from: 0 },
            "S",
            false,
            false,
            false,
            &[Yaku::Yakuhai],
            &[],
            1,
            40,
        ),
        case(
            "yakuhai_red_dragon_ron",
            "yakuhai",
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "6s", "7s", "8s", "3p", "3p", "rd", "rd",
            ],
            &[],
            3,
            0,
            WinType::Ron { from: 0 },
            "rd",
            false,
            false,
            false,
            &[Yaku::Yakuhai],
            &[],
            1,
            40,
        ),
        case(
            "yakuhai_white_dragon_ron",
            "yakuhai",
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "6s", "7s", "8s", "3p", "3p", "wd", "wd",
            ],
            &[],
            2,
            0,
            WinType::Ron { from: 1 },
            "wd",
            false,
            false,
            false,
            &[Yaku::Yakuhai],
            &[],
            1,
            40,
        ),
        case(
            "yakuhai_green_dragon_ron",
            "yakuhai",
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "6s", "7s", "8s", "3p", "3p", "gd", "gd",
            ],
            &[],
            3,
            1,
            WinType::Ron { from: 1 },
            "gd",
            false,
            false,
            false,
            &[Yaku::Yakuhai],
            &[],
            1,
            40,
        ),
        case(
            "yakuhai_menzen_tsumo",
            "yakuhai",
            &[
                "2m", "3m", "4m", "5p", "6p", "7p", "6s", "7s", "8s", "rd", "rd", "2p", "2p",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "2p",
            false,
            false,
            false,
            &[Yaku::Yakuhai, Yaku::MenzenTsumo],
            &[Yaku::Tanyao, Yaku::Pinfu],
            2,
            22,
        ),
        case(
            "riichi_tsumo_tanyao",
            "riichi",
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ],
            &[],
            2,
            0,
            WinType::Tsumo,
            "2p",
            true,
            false,
            false,
            &[Yaku::Riichi, Yaku::MenzenTsumo, Yaku::Tanyao],
            &[],
            3,
            28,
        ),
        case(
            "riichi_ron_tanyao",
            "riichi",
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p",
            ],
            &[],
            1,
            0,
            WinType::Ron { from: 0 },
            "2p",
            true,
            false,
            false,
            &[Yaku::Riichi, Yaku::Tanyao],
            &[Yaku::MenzenTsumo],
            2,
            36,
        ),
        case(
            "chiitoitsu_menzen_tsumo",
            "chiitoitsu",
            &[
                "2m", "2m", "3m", "3m", "4m", "4m", "5p", "5p", "6p", "6p", "7s", "7s", "8s", "8s",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "8s",
            false,
            false,
            false,
            &[Yaku::Chiitoitsu, Yaku::MenzenTsumo, Yaku::Tanyao],
            &[Yaku::Pinfu],
            4,
            25,
        ),
        case(
            "chiitoitsu_tanyao_menzen_tsumo",
            "chiitoitsu",
            &[
                "2m", "2m", "3m", "3m", "4m", "4m", "5p", "5p", "6p", "6p", "7s", "7s", "8s", "8s",
            ],
            &[],
            1,
            0,
            WinType::Tsumo,
            "8s",
            false,
            false,
            false,
            &[Yaku::Chiitoitsu, Yaku::Tanyao, Yaku::MenzenTsumo],
            &[],
            4,
            25,
        ),
    ]
}

/// Win fixtures for Phase 11.5 pattern yaku.
pub fn pattern_cases() -> Vec<WinCase> {
    vec![
        case(
            "toitoi_open_ron",
            "toitoi",
            &["1m", "1m", "1m", "2m", "2m", "2m", "3p", "3p", "3p", "5p"],
            &[("pon", ["4s", "4s", "4s"], "4s")],
            0,
            0,
            WinType::Ron { from: 2 },
            "5p",
            false,
            false,
            false,
            &[Yaku::Toitoi],
            &[Yaku::MenzenTsumo, Yaku::Pinfu],
            2,
            40,
        ),
        case(
            "iipeikou_menzen_tsumo",
            "iipeikou",
            &[
                "1m", "2m", "3m", "1m", "2m", "3m", "4p", "5p", "6p", "7s", "8s", "9s", "1m", "1m",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "1m",
            false,
            false,
            false,
            &[Yaku::Iipeikou, Yaku::MenzenTsumo],
            &[Yaku::Ryanpeikou, Yaku::Pinfu],
            2,
            24,
        ),
        case(
            "ryanpeikou_menzen_tsumo",
            "ryanpeikou",
            &[
                "1m", "2m", "3m", "1m", "2m", "3m", "4p", "5p", "6p", "4p", "5p", "6p", "7s", "7s",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "7s",
            false,
            false,
            false,
            &[Yaku::Ryanpeikou, Yaku::MenzenTsumo],
            &[Yaku::Iipeikou, Yaku::Pinfu, Yaku::Chiitoitsu],
            4,
            24,
        ),
        case(
            "sanshoku_closed_tsumo",
            "sanshoku",
            &[
                "1m", "2m", "3m", "1p", "2p", "3p", "1s", "2s", "3s", "4m", "5m", "6m", "7m", "7m",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "7m",
            false,
            false,
            false,
            &[Yaku::Sanshoku, Yaku::MenzenTsumo],
            &[],
            3,
            24,
        ),
        case(
            "sanshoku_open_ron",
            "sanshoku",
            &["1p", "2p", "3p", "1s", "2s", "3s", "4m", "5m", "6m", "7m"],
            &[("chi", ["1m", "2m", "3m"], "1m")],
            0,
            0,
            WinType::Ron { from: 2 },
            "7m",
            false,
            false,
            false,
            &[Yaku::Sanshoku],
            &[Yaku::MenzenTsumo],
            1,
            30,
        ),
        case(
            "ittsu_closed_tsumo",
            "ittsu",
            &[
                "1m", "2m", "3m", "4m", "5m", "6m", "7m", "8m", "9m", "1p", "1p", "2p", "2p", "2p",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "2p",
            false,
            false,
            false,
            &[Yaku::Ittsu, Yaku::MenzenTsumo],
            &[],
            3,
            28,
        ),
        case(
            "ittsu_open_ron",
            "ittsu",
            &["4m", "5m", "6m", "7m", "8m", "9m", "1p", "1p", "1p", "2p"],
            &[("chi", ["1m", "2m", "3m"], "1m")],
            0,
            0,
            WinType::Ron { from: 2 },
            "2p",
            false,
            false,
            false,
            &[Yaku::Ittsu],
            &[Yaku::MenzenTsumo],
            1,
            30,
        ),
        case(
            "honitsu_tsumo",
            "honitsu",
            &[
                "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "9p", "9p", "E", "E", "2p", "2p",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "2p",
            false,
            false,
            false,
            &[Yaku::Honitsu, Yaku::Yakuhai, Yaku::MenzenTsumo],
            &[Yaku::Chinitsu, Yaku::Tanyao],
            5,
            38,
        ),
        case(
            "chinitsu_tsumo",
            "chinitsu",
            &[
                "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "9p", "9p", "3p", "4p", "5p", "5p",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "5p",
            false,
            false,
            false,
            &[Yaku::Chinitsu, Yaku::MenzenTsumo],
            &[Yaku::Honitsu, Yaku::Yakuhai, Yaku::Tanyao],
            7,
            32,
        ),
        case(
            "chanta_ron",
            "chanta",
            &[
                "1m", "2m", "3m", "7m", "8m", "9m", "1p", "1p", "1p", "9s", "9s", "9s", "E",
            ],
            &[],
            0,
            0,
            WinType::Ron { from: 2 },
            "E",
            false,
            false,
            false,
            &[Yaku::Chanta, Yaku::Yakuhai],
            &[Yaku::Junchan, Yaku::Tanyao],
            3,
            50,
        ),
        case(
            "junchan_tsumo",
            "junchan",
            &[
                "1m", "2m", "3m", "7m", "8m", "9m", "1p", "1p", "1p", "9s", "9s", "9s", "1m", "1m",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "1m",
            false,
            false,
            false,
            &[Yaku::Junchan, Yaku::MenzenTsumo],
            &[Yaku::Chanta, Yaku::Tanyao, Yaku::Honitsu],
            4,
            40,
        ),
    ]
}

/// Win fixtures for Phase 11.6 riichi-timing yaku.
pub fn riichi_timing_cases() -> Vec<WinCase> {
    vec![
        case(
            "ippatsu_tsumo_tanyao",
            "ippatsu",
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ],
            &[],
            2,
            0,
            WinType::Tsumo,
            "2p",
            true,
            true,
            false,
            &[Yaku::Riichi, Yaku::Ippatsu, Yaku::MenzenTsumo, Yaku::Tanyao],
            &[],
            4,
            28,
        ),
        case(
            "double_riichi_tsumo_tanyao",
            "double_riichi",
            &[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ],
            &[],
            2,
            0,
            WinType::Tsumo,
            "2p",
            true,
            false,
            true,
            &[Yaku::DoubleRiichi, Yaku::MenzenTsumo, Yaku::Tanyao],
            &[Yaku::Riichi],
            4,
            28,
        ),
    ]
}

/// Win fixtures for Phase 11.8 yakuman.
pub fn yakuman_cases() -> Vec<WinCase> {
    vec![
        case(
            "kokushi_menzen_tsumo",
            "kokushi",
            &[
                "1m", "9m", "1p", "9p", "1s", "9s", "E", "S", "W", "N", "wd", "gd", "rd", "E",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "E",
            false,
            false,
            false,
            &[Yaku::Kokushi],
            &[
                Yaku::Chiitoitsu,
                Yaku::Toitoi,
                Yaku::Yakuhai,
                Yaku::Tanyao,
                Yaku::MenzenTsumo,
            ],
            13,
            30,
        ),
        case(
            "suuankou_menzen_tsumo",
            "suuankou",
            &[
                "1m", "1m", "1m", "2m", "2m", "2m", "3m", "3m", "3m", "4m", "4m", "4m", "5m", "5m",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "5m",
            false,
            false,
            false,
            &[Yaku::Suuankou],
            &[Yaku::Toitoi, Yaku::MenzenTsumo, Yaku::Tanyao],
            13,
            30,
        ),
        case(
            "daisangen_menzen_tsumo",
            "daisangen",
            &[
                "wd", "wd", "wd", "gd", "gd", "gd", "rd", "rd", "rd", "1m", "2m", "3m", "4m", "4m",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "4m",
            false,
            false,
            false,
            &[Yaku::Daisangen],
            &[
                Yaku::Toitoi,
                Yaku::Yakuhai,
                Yaku::Honitsu,
                Yaku::MenzenTsumo,
            ],
            13,
            30,
        ),
        case(
            "daisangen_open_ron",
            "daisangen",
            &["1m", "2m", "3m", "4m"],
            &[
                ("pon", ["wd", "wd", "wd"], "wd"),
                ("pon", ["gd", "gd", "gd"], "gd"),
                ("pon", ["rd", "rd", "rd"], "rd"),
            ],
            0,
            0,
            WinType::Ron { from: 2 },
            "4m",
            false,
            false,
            false,
            &[Yaku::Daisangen],
            &[Yaku::Toitoi, Yaku::Yakuhai, Yaku::MenzenTsumo],
            13,
            30,
        ),
        case(
            "shousuushii_menzen_tsumo",
            "shousuushii",
            &[
                "E", "E", "E", "S", "S", "S", "W", "W", "W", "1m", "2m", "3m", "N", "N",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "N",
            false,
            false,
            false,
            &[Yaku::Shousuushii],
            &[Yaku::Toitoi, Yaku::Yakuhai, Yaku::MenzenTsumo],
            13,
            30,
        ),
        case(
            "daisuushii_menzen_tsumo",
            "daisuushii",
            &[
                "E", "E", "E", "S", "S", "S", "W", "W", "W", "N", "N", "N", "1m", "1m",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "1m",
            false,
            false,
            false,
            &[Yaku::Daisuushii],
            &[
                Yaku::Shousuushii,
                Yaku::Toitoi,
                Yaku::Yakuhai,
                Yaku::MenzenTsumo,
            ],
            26,
            30,
        ),
        case(
            "chuuren_menzen_tsumo",
            "chuuren",
            &[
                "1m", "1m", "1m", "2m", "3m", "4m", "5m", "6m", "7m", "8m", "9m", "9m", "9m", "5m",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "5m",
            false,
            false,
            false,
            &[Yaku::Chuuren],
            &[Yaku::Chinitsu, Yaku::Honitsu, Yaku::MenzenTsumo],
            13,
            30,
        ),
        case(
            "ryuuiisou_menzen_tsumo",
            "ryuuiisou",
            &[
                "2s", "2s", "2s", "3s", "3s", "3s", "4s", "4s", "4s", "6s", "6s", "6s", "8s", "8s",
            ],
            &[],
            0,
            0,
            WinType::Tsumo,
            "8s",
            false,
            false,
            false,
            &[Yaku::Ryuuiisou],
            &[
                Yaku::Toitoi,
                Yaku::Chiitoitsu,
                Yaku::MenzenTsumo,
                Yaku::Tanyao,
            ],
            13,
            30,
        ),
        WinCase {
            id: "suukantsu_ron",
            cheatsheet_id: "suukantsu",
            concealed: parse_tiles(&["1m"]),
            melds: vec![
                Meld::open_kan(
                    [
                        Tile::from_str("2m").unwrap(),
                        Tile::from_str("2m").unwrap(),
                        Tile::from_str("2m").unwrap(),
                        Tile::from_str("2m").unwrap(),
                    ],
                    Tile::from_str("2m").unwrap(),
                )
                .expect("kan"),
                Meld::open_kan(
                    [
                        Tile::from_str("3m").unwrap(),
                        Tile::from_str("3m").unwrap(),
                        Tile::from_str("3m").unwrap(),
                        Tile::from_str("3m").unwrap(),
                    ],
                    Tile::from_str("3m").unwrap(),
                )
                .expect("kan"),
                Meld::open_kan(
                    [
                        Tile::from_str("4m").unwrap(),
                        Tile::from_str("4m").unwrap(),
                        Tile::from_str("4m").unwrap(),
                        Tile::from_str("4m").unwrap(),
                    ],
                    Tile::from_str("4m").unwrap(),
                )
                .expect("kan"),
                Meld::open_kan(
                    [
                        Tile::from_str("5m").unwrap(),
                        Tile::from_str("5m").unwrap(),
                        Tile::from_str("5m").unwrap(),
                        Tile::from_str("5m").unwrap(),
                    ],
                    Tile::from_str("5m").unwrap(),
                )
                .expect("kan"),
            ],
            winner: 0,
            dealer: 0,
            win_type: WinType::Ron { from: 1 },
            win_tile: Tile::from_str("1m").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: None,
            is_rinshan_draw: false,
            is_dealer_first_turn: false,
            live_draws: [0; 4],
            calls_made: false,
            is_chankan: false,
            must_include: &[Yaku::Suukantsu],
            must_exclude: &[Yaku::MenzenTsumo, Yaku::Toitoi, Yaku::Tanyao],
            expected_yaku_han: 13,
            expected_fu: 30,
        },
    ]
}

pub fn all_win_cases() -> Vec<WinCase> {
    let mut cases = baseline_cases();
    cases.extend(pattern_cases());
    cases.extend(riichi_timing_cases());
    cases.extend(win_timing_cases());
    cases.extend(yakuman_cases());
    cases
}

#[allow(clippy::too_many_arguments)]
fn case(
    id: &'static str,
    cheatsheet_id: &'static str,
    concealed: &[&str],
    melds: &[MeldSpec],
    winner: usize,
    dealer: usize,
    win_type: WinType,
    win_tile: &str,
    riichi: bool,
    ippatsu_live: bool,
    double_riichi: bool,
    must_include: &'static [Yaku],
    must_exclude: &'static [Yaku],
    expected_yaku_han: u8,
    expected_fu: u8,
) -> WinCase {
    WinCase {
        id,
        cheatsheet_id,
        concealed: parse_tiles(concealed),
        melds: melds.iter().map(parse_meld).collect(),
        winner,
        dealer,
        win_type,
        win_tile: Tile::from_str(win_tile).expect("win tile"),
        riichi,
        ippatsu_live,
        double_riichi,
        wall_live_remaining: None,
        is_rinshan_draw: false,
        is_dealer_first_turn: false,
        live_draws: [0; 4],
        calls_made: false,
        is_chankan: false,
        must_include,
        must_exclude,
        expected_yaku_han,
        expected_fu,
    }
}

fn parse_tiles(labels: &[&str]) -> Vec<Tile> {
    labels
        .iter()
        .map(|label| Tile::from_str(label).unwrap_or_else(|_| panic!("tile {label}")))
        .collect()
}

fn parse_meld((kind, tiles, called): &MeldSpec) -> Meld {
    let tiles: [Tile; 3] = [
        Tile::from_str(tiles[0]).unwrap(),
        Tile::from_str(tiles[1]).unwrap(),
        Tile::from_str(tiles[2]).unwrap(),
    ];
    let called = Tile::from_str(called).unwrap();
    match *kind {
        "pon" => Meld::pon(tiles, called).expect("pon meld"),
        "chi" => Meld::chi(tiles, called).expect("chi meld"),
        _ => panic!("unknown meld kind"),
    }
}

pub fn state_for_case(case: &WinCase) -> HandState {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(99));
    let deal = wall.deal(case.dealer).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    let hand = Hand::new(
        Concealed::from_tiles(case.concealed.clone()),
        case.melds.clone(),
    )
    .expect("hand");
    state.set_hand(case.winner, hand);
    if case.riichi {
        state.riichi[case.winner] = true;
    }
    if case.ippatsu_live {
        state.ippatsu_live[case.winner] = true;
    }
    if case.double_riichi {
        state.double_riichi[case.winner] = true;
    }
    state.is_dealer_first_turn = case.is_dealer_first_turn;
    state.is_rinshan_draw = case.is_rinshan_draw;
    state.live_draws = case.live_draws;
    state.calls_made = case.calls_made;
    if case.cheatsheet_id == "renhou" {
        state.first_discards[case.dealer] = Some(case.win_tile);
    }
    if let Some(remaining) = case.wall_live_remaining {
        while state.wall().live_remaining() > remaining {
            state.wall_mut().draw_live().expect("drain live wall");
        }
    }
    state
}

pub fn win_timing_cases() -> Vec<WinCase> {
    vec![
        WinCase {
            id: "haitei_tsumo_tanyao",
            cheatsheet_id: "haitei",
            concealed: parse_tiles(&[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ]),
            melds: vec![],
            winner: 0,
            dealer: 0,
            win_type: WinType::Tsumo,
            win_tile: Tile::from_str("2p").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: Some(0),
            is_rinshan_draw: false,
            is_dealer_first_turn: false,
            live_draws: [1, 0, 0, 0],
            calls_made: false,
            is_chankan: false,
            must_include: &[Yaku::HaiteiHoutei, Yaku::MenzenTsumo, Yaku::Tanyao],
            must_exclude: &[],
            expected_yaku_han: 3,
            expected_fu: 28,
        },
        WinCase {
            id: "houtei_ron_tanyao",
            cheatsheet_id: "haitei",
            concealed: parse_tiles(&[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p",
            ]),
            melds: vec![],
            winner: 1,
            dealer: 0,
            win_type: WinType::Ron { from: 0 },
            win_tile: Tile::from_str("2p").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: Some(0),
            is_rinshan_draw: false,
            is_dealer_first_turn: false,
            live_draws: [0; 4],
            calls_made: false,
            is_chankan: false,
            must_include: &[Yaku::HaiteiHoutei, Yaku::Tanyao],
            must_exclude: &[Yaku::MenzenTsumo],
            expected_yaku_han: 2,
            expected_fu: 36,
        },
        WinCase {
            id: "rinshan_tsumo_tanyao",
            cheatsheet_id: "rinshan",
            concealed: parse_tiles(&[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ]),
            melds: vec![],
            winner: 0,
            dealer: 0,
            win_type: WinType::Tsumo,
            win_tile: Tile::from_str("2p").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: None,
            is_rinshan_draw: true,
            is_dealer_first_turn: false,
            live_draws: [0; 4],
            calls_made: false,
            is_chankan: false,
            must_include: &[Yaku::Rinshan, Yaku::MenzenTsumo, Yaku::Tanyao],
            must_exclude: &[],
            expected_yaku_han: 3,
            expected_fu: 28,
        },
        WinCase {
            id: "chankan_ron_tanyao",
            cheatsheet_id: "chankan",
            concealed: parse_tiles(&[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "2p", "2p",
            ]),
            melds: vec![],
            winner: 1,
            dealer: 0,
            win_type: WinType::Ron { from: 0 },
            win_tile: Tile::from_str("5s").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: None,
            is_rinshan_draw: false,
            is_dealer_first_turn: false,
            live_draws: [0; 4],
            calls_made: false,
            is_chankan: true,
            must_include: &[Yaku::Chankan, Yaku::Tanyao],
            must_exclude: &[Yaku::MenzenTsumo],
            expected_yaku_han: 2,
            expected_fu: 36,
        },
        WinCase {
            id: "renhou_ron_tanyao",
            cheatsheet_id: "renhou",
            concealed: parse_tiles(&[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p",
            ]),
            melds: vec![],
            winner: 1,
            dealer: 0,
            win_type: WinType::Ron { from: 0 },
            win_tile: Tile::from_str("2p").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: None,
            is_rinshan_draw: false,
            is_dealer_first_turn: false,
            live_draws: [0, 0, 0, 0],
            calls_made: false,
            is_chankan: false,
            must_include: &[Yaku::Renhou, Yaku::Tanyao],
            must_exclude: &[Yaku::MenzenTsumo],
            expected_yaku_han: 6,
            expected_fu: 36,
        },
        WinCase {
            id: "tenhou_tsumo_tanyao",
            cheatsheet_id: "tenhou",
            concealed: parse_tiles(&[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ]),
            melds: vec![],
            winner: 0,
            dealer: 0,
            win_type: WinType::Tsumo,
            win_tile: Tile::from_str("2p").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: None,
            is_rinshan_draw: false,
            is_dealer_first_turn: true,
            live_draws: [0; 4],
            calls_made: false,
            is_chankan: false,
            must_include: &[Yaku::TenhouChiihou, Yaku::Tanyao],
            must_exclude: &[Yaku::MenzenTsumo, Yaku::HaiteiHoutei],
            expected_yaku_han: 14,
            expected_fu: 30,
        },
        WinCase {
            id: "chiihou_tsumo_tanyao",
            cheatsheet_id: "tenhou",
            concealed: parse_tiles(&[
                "2m", "3m", "4m", "3p", "4p", "5p", "6s", "7s", "8s", "5s", "5s", "5s", "2p", "2p",
            ]),
            melds: vec![],
            winner: 1,
            dealer: 0,
            win_type: WinType::Tsumo,
            win_tile: Tile::from_str("2p").unwrap(),
            riichi: false,
            ippatsu_live: false,
            double_riichi: false,
            wall_live_remaining: None,
            is_rinshan_draw: false,
            is_dealer_first_turn: false,
            live_draws: [0, 1, 0, 0],
            calls_made: false,
            is_chankan: false,
            must_include: &[Yaku::TenhouChiihou, Yaku::MenzenTsumo, Yaku::Tanyao],
            must_exclude: &[Yaku::HaiteiHoutei],
            expected_yaku_han: 15,
            expected_fu: 30,
        },
    ]
}

pub fn score_case(case: &WinCase) -> crate::scoring::ScoringResult {
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();
    let state = state_for_case(case);
    let ctx = WinContext::new(
        &state,
        case.winner,
        case.win_type,
        case.win_tile,
        WinTimingFlags {
            is_chankan: case.is_chankan,
        },
    );
    assert!(profile.can_win(&ctx, &config), "case {} must win", case.id);
    profile.score_win(&ctx, &config)
}

#[test]
fn pattern_win_combinations_score_expected_yaku_fu_and_han() {
    assert_win_cases(pattern_cases());
}

#[test]
fn win_timing_win_combinations_score_expected_yaku_fu_and_han() {
    assert_win_cases(win_timing_cases());
}

#[test]
fn riichi_timing_win_combinations_score_expected_yaku_fu_and_han() {
    assert_win_cases(riichi_timing_cases());
}

#[test]
fn baseline_win_combinations_score_expected_yaku_fu_and_han() {
    assert_win_cases(baseline_cases());
}

#[test]
fn yakuman_win_combinations_score_expected_yaku_fu_and_han() {
    assert_win_cases(yakuman_cases());
}

fn assert_win_cases(cases: Vec<WinCase>) {
    for case in cases {
        let result = score_case(&case);
        for yaku in case.must_include {
            assert!(
                result.yaku.contains(yaku),
                "{}: expected {:?} in {:?}",
                case.id,
                yaku,
                result.yaku
            );
        }
        for yaku in case.must_exclude {
            assert!(
                !result.yaku.contains(yaku),
                "{}: did not expect {:?} in {:?}",
                case.id,
                yaku,
                result.yaku
            );
        }
        let is_open = !case.melds.is_empty();
        assert_eq!(
            total_han(&result.yaku, is_open),
            case.expected_yaku_han,
            "{}: yaku han",
            case.id
        );
        assert_eq!(result.fu, case.expected_fu, "{}: fu", case.id);
        assert!(
            result.han >= case.expected_yaku_han,
            "{}: total han {} < yaku han {}",
            case.id,
            result.han,
            case.expected_yaku_han
        );
    }
}

#[test]
fn every_implemented_cheatsheet_row_has_win_fixture() {
    let covered: std::collections::HashSet<_> =
        all_win_cases().iter().map(|c| c.cheatsheet_id).collect();
    for row in cheatsheet::implemented_rows() {
        assert!(
            covered.contains(row.id),
            "missing win fixture for cheatsheet row {}",
            row.id
        );
    }
}

fn win_types_match(a: WinType, b: WinType) -> bool {
    match (a, b) {
        (WinType::Tsumo, WinType::Tsumo) => true,
        (WinType::Ron { from: x }, WinType::Ron { from: y }) => x == y,
        _ => false,
    }
}

#[test]
fn candidate_win_paths_match_score_win_on_fixtures() {
    let config = RulesConfig::standard();
    for case in all_win_cases() {
        if case.is_chankan {
            continue;
        }
        let state = state_for_case(&case);
        let paths = crate::rules::candidate_win_paths(&state, case.winner, &config, 8);
        assert!(
            !paths.is_empty(),
            "case {} should have at least one candidate path",
            case.id
        );
        let expected = score_case(&case);
        let best = paths
            .iter()
            .find(|path| {
                win_types_match(path.win_type, case.win_type)
                    && path.han == expected.han
                    && path.fu == expected.fu
                    && path.expected_points == expected.deltas[case.winner]
            })
            .unwrap_or_else(|| {
                panic!(
                    "case {}: no matching path for {:?} ({} han {} fu +{}), got {} paths",
                    case.id,
                    case.win_type,
                    expected.han,
                    expected.fu,
                    expected.deltas[case.winner],
                    paths.len()
                )
            });
        for yaku in case.must_include {
            assert!(
                best.yaku.contains(yaku),
                "case {}: expected {:?} in {:?}",
                case.id,
                yaku,
                best.yaku
            );
        }
    }
}

#[test]
fn cheatsheet_catalog_covers_reference_yaku() {
    assert!(
        cheatsheet::ROWS.len() >= 25,
        "cheatsheet catalog should list all reference yaku families"
    );
    let impl_count = cheatsheet::implemented_rows().count();
    assert_eq!(
        impl_count, 30,
        "all cheatsheet yaku families are implemented through Phase 11.8"
    );
}
