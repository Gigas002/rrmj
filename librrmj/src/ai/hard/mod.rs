use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::action::Action;
use crate::agent::{Agent, PlayerView};
use crate::state::HandPhase;

use super::common::{hand_from_view, prefer_win, simulate_call};
use super::defense::{any_opponent_riichi, tile_safety};
use super::efficiency::weighted_waiting_count;
use super::shanten::{best_waiting_potential, hand_without_concealed_tile, waiting_count};

// --- Tunable strategy constants (see PLAN Phase 8) ---

/// Minimum plain ukeire before declaring riichi.
const MIN_PLAIN_UKEIRE_FOR_RIICHI: usize = 2;

/// Minimum points in hand before paying the 1,000-point riichi stick.
const MIN_POINTS_FOR_RIICHI: i32 = 1_000;

/// Hard agent: medium baseline plus weighted tie-breaks, riichi timing, and suji/genbutsu defense.
#[derive(Debug)]
pub struct HardAgent {
    rng: StdRng,
}

impl HardAgent {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn is_tenpai_shape(hand: &crate::hand::Hand) -> bool {
        if hand.concealed().len() == 14 {
            best_waiting_potential(hand) > 0
        } else {
            waiting_count(hand) > 0
        }
    }

    fn should_defend(view: &PlayerView, hand: &crate::hand::Hand) -> bool {
        any_opponent_riichi(view) && !Self::is_tenpai_shape(hand)
    }

    fn discard_candidates(legal: &[Action], can_riichi: bool) -> Vec<Action> {
        let mut candidates: Vec<Action> = legal
            .iter()
            .copied()
            .filter(|action| matches!(action, Action::Discard(_)))
            .collect();
        if can_riichi {
            candidates.extend(
                legal
                    .iter()
                    .copied()
                    .filter(|action| matches!(action, Action::Riichi { .. })),
            );
        }
        candidates
    }

    fn score_discard(
        view: &PlayerView,
        hand: &crate::hand::Hand,
        action: Action,
        defending: bool,
    ) -> Option<(usize, u32, u16)> {
        let tile = match action {
            Action::Discard(tile) | Action::Riichi { discard: tile } => tile,
            _ => return None,
        };
        let after = hand_without_concealed_tile(hand, tile)?;
        let plain = waiting_count(&after);
        if matches!(action, Action::Riichi { .. }) && plain < MIN_PLAIN_UKEIRE_FOR_RIICHI {
            return None;
        }
        let weighted = weighted_waiting_count(&after, view);
        let safety = if defending {
            tile_safety(view, tile)
        } else {
            0
        };
        Some((plain, weighted, safety))
    }

    fn choose_discard(rng: &mut impl Rng, view: &PlayerView, legal: &[Action]) -> Action {
        let Some(hand) = hand_from_view(view) else {
            return legal[0];
        };

        let defending = Self::should_defend(view, &hand);
        let can_riichi = !defending
            && hand.melds().is_empty()
            && view.scores[view.seat] >= MIN_POINTS_FOR_RIICHI;

        let candidates = Self::discard_candidates(legal, can_riichi);
        if candidates.is_empty() {
            return *legal
                .choose(rng)
                .expect("legal discard actions expected in discard phase");
        }

        let mut ranked: Vec<(Action, usize, u32, u16)> = Vec::new();
        for action in candidates {
            if let Some((plain, weighted, safety)) =
                Self::score_discard(view, &hand, action, defending)
            {
                ranked.push((action, plain, weighted, safety));
            }
        }

        if ranked.is_empty() {
            return *legal
                .iter()
                .find(|action| matches!(action, Action::Discard(_)))
                .expect("discard fallback");
        }

        let best_plain = ranked.iter().map(|(_, plain, _, _)| *plain).max().unwrap();
        ranked.retain(|(_, plain, _, _)| *plain == best_plain);

        if defending {
            let best_weighted = ranked.iter().map(|(_, _, w, _)| *w).max().unwrap();
            ranked.retain(|(_, _, w, _)| *w == best_weighted);
            ranked.sort_by_key(|(_, _, _, safety)| std::cmp::Reverse(*safety));
            return ranked[0].0;
        }

        let best_weighted = ranked.iter().map(|(_, _, w, _)| *w).max().unwrap();
        let mut top: Vec<Action> = ranked
            .iter()
            .filter(|(_, _, w, _)| *w == best_weighted)
            .map(|(action, _, _, _)| *action)
            .collect();

        if top.is_empty() {
            top = ranked.iter().map(|(action, _, _, _)| *action).collect();
        }

        *top.choose(rng)
            .expect("at least one discard candidate should score")
    }

    fn choose_reaction(_rng: &mut impl Rng, view: &PlayerView, legal: &[Action]) -> Action {
        if let Some(win) = prefer_win(legal) {
            return win;
        }

        let Some(hand) = hand_from_view(view) else {
            return Action::Pass;
        };
        let baseline = best_waiting_potential(&hand);

        let Some(pending) = view.pending_call else {
            return Action::Pass;
        };

        let mut best = Action::Pass;
        let mut best_plain = baseline;
        let mut best_weighted = 0u32;

        for action in legal {
            if matches!(action, Action::Pass | Action::Ron) {
                continue;
            }
            let Some(after) = simulate_call(&hand, *action, pending.tile) else {
                continue;
            };
            let plain = best_waiting_potential(&after);
            let weighted = weighted_waiting_count(&after, view);
            let better = plain > best_plain || (plain == best_plain && weighted > best_weighted);
            if better {
                best_plain = plain;
                best_weighted = weighted;
                best = *action;
            }
        }

        if best_plain > baseline {
            best
        } else {
            Action::Pass
        }
    }
}

impl Agent for HardAgent {
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action {
        if legal.is_empty() {
            panic!("HardAgent asked to decide with no legal actions");
        }

        if let Some(win) = prefer_win(legal) {
            return win;
        }

        match view.phase {
            HandPhase::Reaction => Self::choose_reaction(&mut self.rng, view, legal),
            HandPhase::Discard => Self::choose_discard(&mut self.rng, view, legal),
            _ => *legal
                .choose(&mut self.rng)
                .expect("legal actions must be non-empty"),
        }
    }
}
