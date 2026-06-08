use crate::rules::RulesConfig;
use crate::rules::profile_trait::WinContext;
use crate::rules::standard::win;
use crate::scoring::{WinType, Yaku};
use crate::tile::{Dragon, Tile, Wind};

pub fn detect_yaku(ctx: &WinContext<'_>, _config: &RulesConfig) -> Vec<Yaku> {
    let mut yaku = Vec::new();

    if ctx.is_riichi() {
        yaku.push(Yaku::Riichi);
    }
    if matches!(ctx.win_type, WinType::Tsumo) && ctx.is_menzen() {
        yaku.push(Yaku::MenzenTsumo);
    }
    if win::is_tanyao_hand(ctx.hand(), ctx.win_tile) {
        yaku.push(Yaku::Tanyao);
    }
    if ctx.is_menzen()
        && win::is_pinfu_hand(ctx.hand(), ctx.win_tile)
        && !has_yakuhai(ctx)
        && yaku.iter().all(|y| *y != Yaku::Tanyao)
    {
        yaku.push(Yaku::Pinfu);
    }
    if has_yakuhai(ctx) {
        yaku.push(Yaku::Yakuhai);
    }

    yaku
}

pub fn total_han(yaku: &[Yaku]) -> u8 {
    yaku.iter().map(|y| y.han()).sum()
}

fn has_yakuhai(ctx: &WinContext<'_>) -> bool {
    let seat_wind = seat_wind_tile(ctx.winner, ctx.state.dealer());
    let round_wind = Tile::wind(Wind::East);
    let dragons = [
        Tile::dragon(Dragon::White),
        Tile::dragon(Dragon::Green),
        Tile::dragon(Dragon::Red),
    ];

    win::hand_contains_yakuhai(ctx.hand(), seat_wind)
        || win::hand_contains_yakuhai(ctx.hand(), round_wind)
        || dragons
            .iter()
            .any(|tile| win::hand_contains_yakuhai(ctx.hand(), *tile))
}

fn seat_wind_tile(seat: usize, dealer: usize) -> Tile {
    let index = (seat + 4 - dealer) % 4;
    Tile::wind(Wind::ALL[index])
}
