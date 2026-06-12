#[cfg(test)]
mod tests;

mod yakuman;

use crate::rules::RulesConfig;
use crate::rules::profile::WinContext;
use crate::rules::standard::patterns;
use crate::rules::standard::win;
use crate::scoring::{WinType, Yaku};

pub use yakuman::is_kokushi_tiles;
use crate::tile::{Dragon, Tile, Wind};

pub fn detect_yaku(ctx: &WinContext<'_>, _config: &RulesConfig) -> Vec<Yaku> {
    let mut yaku = Vec::new();

    if ctx.state.is_double_riichi(ctx.winner) {
        yaku.push(Yaku::DoubleRiichi);
    } else if ctx.is_riichi() {
        yaku.push(Yaku::Riichi);
    }
    if ctx.is_riichi() && ctx.state.ippatsu_live(ctx.winner) {
        yaku.push(Yaku::Ippatsu);
    }
    if ctx.is_tenhou() || ctx.is_chiihou() {
        yaku.push(Yaku::TenhouChiihou);
    } else {
        if ctx.is_haitei || ctx.is_houtei {
            yaku.push(Yaku::HaiteiHoutei);
        }
        if ctx.is_rinshan {
            yaku.push(Yaku::Rinshan);
        }
        if ctx.is_chankan {
            yaku.push(Yaku::Chankan);
        }
        if ctx.is_renhou() {
            yaku.push(Yaku::Renhou);
        }
    }
    let yakuman = yakuman::detect(ctx);
    if !yakuman.is_empty() {
        yaku.extend(yakuman);
        return yaku;
    }

    if matches!(ctx.win_type, WinType::Tsumo) && ctx.is_menzen() && !ctx.is_tenhou() {
        yaku.push(Yaku::MenzenTsumo);
    }
    if patterns::is_chinitsu_hand(ctx) {
        yaku.push(Yaku::Chinitsu);
    } else if patterns::is_honitsu_hand(ctx) {
        yaku.push(Yaku::Honitsu);
    }
    if patterns::is_junchan_hand(ctx) {
        yaku.push(Yaku::Junchan);
    } else if patterns::is_chanta_hand(ctx) {
        yaku.push(Yaku::Chanta);
    }
    if win::is_tanyao_hand(ctx.hand(), ctx.win_tile, ctx.win_type) {
        yaku.push(Yaku::Tanyao);
    }
    if patterns::is_ryanpeikou_hand(ctx) {
        yaku.push(Yaku::Ryanpeikou);
    } else if patterns::is_iipeikou_hand(ctx) {
        yaku.push(Yaku::Iipeikou);
    }
    if patterns::is_toitoi_hand(ctx) {
        yaku.push(Yaku::Toitoi);
    }
    if patterns::is_sanshoku_hand(ctx) {
        yaku.push(Yaku::Sanshoku);
    }
    if patterns::is_ittsu_hand(ctx) {
        yaku.push(Yaku::Ittsu);
    }
    if ctx.is_menzen()
        && win::is_pinfu_hand(ctx)
        && !has_yakuhai(ctx)
        && yaku.iter().all(|y| *y != Yaku::Tanyao)
        && !yaku.iter().any(|y| {
            matches!(
                y,
                Yaku::Toitoi | Yaku::Iipeikou | Yaku::Ryanpeikou | Yaku::Sanshoku | Yaku::Ittsu
            )
        })
    {
        yaku.push(Yaku::Pinfu);
    }
    if has_yakuhai(ctx) {
        yaku.push(Yaku::Yakuhai);
    }
    if ctx.is_menzen()
        && win::is_chiitoitsu_hand(ctx.hand(), ctx.win_tile, ctx.win_type)
        && !standard_form_pattern_yaku(ctx)
    {
        yaku.push(Yaku::Chiitoitsu);
    }

    yaku
}

pub fn han_for_yaku(yaku: Yaku, is_open: bool) -> u8 {
    let base = yaku.closed_han();
    if is_open && yaku.open_han_penalty() {
        base.saturating_sub(1)
    } else {
        base
    }
}

pub fn total_han(yaku: &[Yaku], is_open: bool) -> u8 {
    yaku.iter().map(|y| han_for_yaku(*y, is_open)).sum()
}

/// Standard-form pattern yaku supersede chiitoitsu when the same tiles qualify for both.
fn standard_form_pattern_yaku(ctx: &WinContext<'_>) -> bool {
    patterns::is_toitoi_hand(ctx)
        || patterns::is_ryanpeikou_hand(ctx)
        || patterns::is_iipeikou_hand(ctx)
        || patterns::is_sanshoku_hand(ctx)
        || patterns::is_ittsu_hand(ctx)
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
