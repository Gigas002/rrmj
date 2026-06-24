#[cfg(test)]
mod tests;

use crate::hand::{KanForm, MeldKind};
use crate::rules::RulesConfig;
use crate::rules::profile::WinContext;
use crate::rules::standard::win;
use crate::scoring::{WinType, Yaku};

pub fn calculate_fu(ctx: &WinContext<'_>, yaku: &[Yaku], config: &RulesConfig) -> u8 {
    if yaku.iter().any(|y| y.is_yakuman()) {
        return 30;
    }

    if yaku.contains(&Yaku::Chiitoitsu) {
        return apply_kiriage(25, config);
    }

    let is_pinfu = yaku.contains(&Yaku::Pinfu);
    if is_pinfu {
        return match ctx.win_type {
            WinType::Ron { .. } => 30,
            WinType::Tsumo => 20,
        };
    }

    let mut fu = 20u8;

    for meld in ctx.hand().melds() {
        fu = fu.saturating_add(open_meld_fu(meld.kind(), meld.tiles()[0]));
    }

    let decomp = win::best_fu_decomposition(ctx);
    fu = fu.saturating_add(decomp.concealed_meld_fu);
    fu = fu.saturating_add(decomp.pair_fu);
    fu = fu.saturating_add(decomp.wait_fu);

    if ctx.is_menzen() && matches!(ctx.win_type, WinType::Ron { .. }) {
        fu = fu.saturating_add(10);
    }

    if matches!(ctx.win_type, WinType::Tsumo) {
        fu = fu.saturating_add(2);
    }

    fu = apply_kiriage(fu, config);

    if !ctx.is_menzen() || matches!(ctx.win_type, WinType::Ron { .. }) {
        fu = fu.max(30);
    }

    fu
}

fn open_meld_fu(kind: MeldKind, tile: crate::tile::Tile) -> u8 {
    match kind {
        MeldKind::Chi => 0,
        MeldKind::Pon => win::triplet_fu(tile, false),
        MeldKind::Kan(KanForm::Open) => win::triplet_fu(tile, false).saturating_mul(4),
        MeldKind::Kan(KanForm::Closed) => win::triplet_fu(tile, true).saturating_mul(4),
    }
}

fn apply_kiriage(fu: u8, config: &RulesConfig) -> u8 {
    if config.kiriage { round_up(fu, 10) } else { fu }
}

fn round_up(value: u8, step: u8) -> u8 {
    value.div_ceil(step) * step
}
