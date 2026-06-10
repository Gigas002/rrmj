#[cfg(test)]
mod tests;

use crate::rules::{RulesConfig, RulesRegistry, WinContext};
use crate::scoring::{Payment, ScoringResult, WinType, Yaku};
use crate::state::HandState;

/// Limit band for basic points before ron/tsumo multipliers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LimitBand {
    Normal,
    Mangan,
    Haneman,
    Baiman,
    Sanbaiman,
    Yakuman,
}

/// Basic points after mangan thresholds (before payment multipliers).
pub(crate) fn base_points(han: u8, fu: u8) -> i32 {
    match limit_band(han, fu) {
        LimitBand::Normal => fu as i32 * 2i32.pow((han + 2) as u32),
        LimitBand::Mangan => 2_000,
        LimitBand::Haneman => 3_000,
        LimitBand::Baiman => 4_000,
        LimitBand::Sanbaiman => 6_000,
        LimitBand::Yakuman => 8_000,
    }
}

fn limit_band(han: u8, fu: u8) -> LimitBand {
    if han >= 13 {
        LimitBand::Yakuman
    } else if han >= 11 {
        LimitBand::Sanbaiman
    } else if han >= 8 {
        LimitBand::Baiman
    } else if han >= 6 {
        LimitBand::Haneman
    } else if han >= 5 || (han == 4 && fu >= 40) || (han == 3 && fu >= 70) {
        LimitBand::Mangan
    } else {
        LimitBand::Normal
    }
}

#[allow(clippy::too_many_arguments)]
pub fn score_hand(
    ctx: &WinContext<'_>,
    yaku: &[Yaku],
    han: u8,
    fu: u8,
    dora: u8,
    ura_dora: u8,
    aka_dora: u8,
    config: &RulesConfig,
) -> ScoringResult {
    let base = base_points(han, fu);
    let mut deltas = [0i32; 4];
    let mut payments = Vec::new();
    let winner = ctx.winner;
    let dealer = ctx.state.dealer();
    let winner_is_dealer = winner == dealer;
    let honba = ctx.state.honba() as i32;
    let riichi_sticks = ctx.state.table_riichi_sticks() as i32 * 1_000;

    match ctx.win_type {
        WinType::Ron { from } => {
            let multiplier = if winner_is_dealer { 6 } else { 4 };
            let total = round_points(base * multiplier) + honba * 300 + riichi_sticks;
            payments.push(Payment {
                payer: from,
                payee: winner,
                points: total,
            });
            deltas[from] -= total;
            deltas[winner] += total;
        }
        WinType::Tsumo => {
            for seat in 0..4 {
                if seat == winner {
                    continue;
                }
                let share = tsumo_share(base, winner_is_dealer, seat == dealer);
                let pay = round_points(share) + honba * 100;
                payments.push(Payment {
                    payer: seat,
                    payee: winner,
                    points: pay,
                });
                deltas[seat] -= pay;
                deltas[winner] += pay;
            }
            deltas[winner] += riichi_sticks;
        }
    }

    let _ = config;

    ScoringResult {
        winner,
        win_type: ctx.win_type,
        yaku: yaku.to_vec(),
        dora,
        ura_dora,
        aka_dora,
        han,
        fu,
        payments,
        deltas,
    }
}

fn tsumo_share(base: i32, winner_is_dealer: bool, payer_is_dealer: bool) -> i32 {
    if winner_is_dealer || payer_is_dealer {
        base * 2
    } else {
        base
    }
}

pub fn score_exhaustive_draw(state: &HandState) -> [i32; 4] {
    let profile = RulesRegistry::get(state.config().profile).expect("standard profile");
    let mut tenpai = [false; 4];
    for (seat, is_tenpai) in tenpai.iter_mut().enumerate() {
        *is_tenpai = profile.is_tenpai(state.hand(seat), state.config());
    }

    let tenpai_count = tenpai.iter().filter(|t| **t).count();
    if tenpai_count == 0 {
        return [0; 4];
    }

    let mut deltas = [0i32; 4];
    let noten_count = 4 - tenpai_count;
    let payment_per_noten = 3_000 / noten_count as i32;
    let gain_per_tenpai = 3_000 / tenpai_count as i32;

    for (seat, is_tenpai) in tenpai.iter().enumerate() {
        if *is_tenpai {
            deltas[seat] += gain_per_tenpai;
        } else {
            deltas[seat] -= payment_per_noten;
        }
    }
    deltas
}

fn round_points(points: i32) -> i32 {
    ((points + 99) / 100) * 100
}
