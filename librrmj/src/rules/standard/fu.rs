use crate::hand::MeldKind;
use crate::rules::profile_trait::WinContext;
use crate::rules::RulesConfig;
use crate::scoring::{WinType, Yaku};

pub fn calculate_fu(ctx: &WinContext<'_>, yaku: &[Yaku], config: &RulesConfig) -> u8 {
    if yaku.contains(&Yaku::Pinfu) && matches!(ctx.win_type, WinType::Ron { .. }) {
        return 30;
    }

    let mut fu = match ctx.win_type {
        WinType::Ron { .. } if ctx.is_menzen() => 30,
        WinType::Ron { .. } => 20,
        WinType::Tsumo => 30,
    };

    if ctx.is_menzen() && matches!(ctx.win_type, WinType::Ron { .. }) {
        fu += 10;
    }

    for meld in ctx.hand().melds() {
        fu += match meld.kind() {
            MeldKind::Chi => 0,
            MeldKind::Pon => 2,
            MeldKind::OpenKan => 8,
            MeldKind::ClosedKan => 8,
            MeldKind::AddedKan => 4,
        };
    }

    if config.kiriage {
        fu = round_up(fu, 10);
    }
    fu.max(20)
}

fn round_up(value: u8, step: u8) -> u8 {
    value.div_ceil(step) * step
}
