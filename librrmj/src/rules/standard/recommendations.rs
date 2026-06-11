use crate::hand::{Concealed, Hand};
use crate::rules::profile_trait::{RulesProfile, WinContext, WinTimingFlags};
use crate::rules::win_path::WinPathCandidate;
use crate::rules::{RulesConfig, RulesRegistry};
use crate::scoring::{ScoringResult, WinType};
use crate::state::HandState;
use crate::tile::Tile;

use super::win::{is_tenpai, is_winning_hand};

pub fn candidate_win_paths(
    state: &HandState,
    seat: usize,
    config: &RulesConfig,
    limit: usize,
) -> Vec<WinPathCandidate> {
    let Ok(profile) = RulesRegistry::get(config.profile) else {
        return Vec::new();
    };
    profile.candidate_win_paths(state, seat, config, limit)
}

struct PathCollector<'a> {
    paths: &'a mut Vec<WinPathCandidate>,
    state: &'a HandState,
    seat: usize,
    config: &'a RulesConfig,
    profile: &'a dyn RulesProfile,
}

impl PathCollector<'_> {
    fn push(&mut self, win_type: WinType, win_tile: Tile, shanten: i8, wait_count: usize) {
        let timing = timing_flags(self.state, win_type);
        let ctx = WinContext::new(
            self.state,
            self.seat,
            win_type,
            win_tile,
            timing,
        );
        if !self.profile.can_win(&ctx, self.config) {
            return;
        }
        let result = self.profile.score_win(&ctx, self.config);
        self.paths.push(candidate_from_result(
            result,
            shanten,
            wait_count,
            Some(win_tile),
        ));
    }
}

pub(crate) fn collect_win_paths(
    state: &HandState,
    seat: usize,
    config: &RulesConfig,
) -> Vec<WinPathCandidate> {
    let Ok(profile) = RulesRegistry::get(config.profile) else {
        return Vec::new();
    };
    let hand = state.hand(seat);
    let mut paths = Vec::new();
    let mut collector = PathCollector {
        paths: &mut paths,
        state,
        seat,
        config,
        profile,
    };

    if is_winning_hand(hand, None) {
        for win_tile in wait_tiles(hand) {
            collector.push(WinType::Tsumo, win_tile, -1, 0);
        }
    }

    if is_tenpai(hand) {
        let waits = wait_tiles(hand);
        let wait_count = waits.len();
        for tile in waits {
            collector.push(WinType::Tsumo, tile, 0, wait_count);
            for from in 0..4 {
                if from == seat {
                    continue;
                }
                collector.push(WinType::Ron { from }, tile, 0, wait_count);
            }
        }
    }

    if hand.concealed().len() % 3 == 2 && !is_winning_hand(hand, None) {
        for discard in unique_concealed_tiles(hand) {
            let Some(after) = hand_after_discard(hand, discard) else {
                continue;
            };
            if !is_tenpai(&after) {
                continue;
            }
            let mut state_after = state.clone();
            state_after.replace_hand(seat, after);
            let waits = wait_tiles(state_after.hand(seat));
            let wait_count = waits.len();
            let mut after_collector = PathCollector {
                paths: &mut paths,
                state: &state_after,
                seat,
                config,
                profile,
            };
            for tile in waits {
                after_collector.push(WinType::Tsumo, tile, 1, wait_count);
            }
        }
    }

    dedupe_paths(&mut paths);
    paths
}

fn candidate_from_result(
    result: ScoringResult,
    shanten: i8,
    wait_count: usize,
    win_tile: Option<Tile>,
) -> WinPathCandidate {
    WinPathCandidate {
        shanten,
        wait_count,
        win_tile,
        yaku: result.yaku,
        han: result.han,
        fu: result.fu,
        dora: result.dora,
        ura_dora: result.ura_dora,
        aka_dora: result.aka_dora,
        expected_points: result.deltas[result.winner],
        win_type: result.win_type,
    }
}

fn timing_flags(state: &HandState, win_type: WinType) -> WinTimingFlags {
    let is_chankan =
        matches!(win_type, WinType::Ron { .. }) && state.is_chankan_window();
    WinTimingFlags { is_chankan }
}

fn dedupe_paths(paths: &mut Vec<WinPathCandidate>) {
    let mut seen = Vec::new();
    paths.retain(|path| {
        let key = (
            path.shanten,
            path.win_type,
            path.han,
            path.fu,
            path.yaku.clone(),
        );
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}

fn wait_tiles(hand: &Hand) -> Vec<Tile> {
    super::win::all_wait_tiles()
        .into_iter()
        .filter(|tile| is_winning_hand(hand, Some(*tile)))
        .collect()
}

fn hand_after_discard(hand: &Hand, discard: Tile) -> Option<Hand> {
    let mut concealed = hand.concealed().tiles().to_vec();
    let pos = concealed.iter().position(|t| *t == discard)?;
    concealed.remove(pos);
    Hand::new(Concealed::from_tiles(concealed), hand.melds().to_vec()).ok()
}

fn unique_concealed_tiles(hand: &Hand) -> Vec<Tile> {
    let mut seen = Vec::new();
    for tile in hand.concealed().tiles() {
        if !seen.contains(tile) {
            seen.push(*tile);
        }
    }
    seen
}

#[cfg(test)]
mod tests;
