use crate::rules::win_path::{WinPathCandidate, sort_win_paths};
use crate::scoring::{WinType, Yaku};

#[test]
fn sort_prefers_higher_expected_points() {
    let mut paths = vec![
        WinPathCandidate {
            shanten: 0,
            wait_count: 4,
            win_tile: None,
            yaku: vec![Yaku::Tanyao],
            han: 1,
            fu: 30,
            dora: 0,
            ura_dora: 0,
            aka_dora: 0,
            expected_points: 1_000,
            win_type: WinType::Ron { from: 0 },
        },
        WinPathCandidate {
            shanten: 0,
            wait_count: 2,
            win_tile: None,
            yaku: vec![Yaku::Pinfu, Yaku::MenzenTsumo],
            han: 2,
            fu: 20,
            dora: 0,
            ura_dora: 0,
            aka_dora: 0,
            expected_points: 2_000,
            win_type: WinType::Tsumo,
        },
    ];
    sort_win_paths(&mut paths);
    assert_eq!(paths[0].expected_points, 2_000);
}
