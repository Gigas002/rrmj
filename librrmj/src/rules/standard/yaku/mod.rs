#[cfg(test)]
mod tests;

mod yakuman;

use crate::rules::RulesConfig;
use crate::rules::profile::WinContext;
use crate::scoring::Yaku;

pub use yakuman::is_kokushi_tiles;

/// Yaku of the best reading of the winning hand (empty when none has a yaku).
pub fn detect_yaku(ctx: &WinContext<'_>, config: &RulesConfig) -> Vec<Yaku> {
    super::eval::evaluate(ctx, config, 0)
        .map(|eval| eval.yaku)
        .unwrap_or_default()
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
