use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::action::Action;
use crate::agent::{Agent, PlayerView};
use crate::state::HandPhase;

use super::common::{hand_from_view, prefer_win, simulate_call};
use super::shanten::{best_waiting_potential, hand_without_concealed_tile, waiting_count};

/// Shanten-guided discards and basic call acceptance.
#[derive(Debug)]
pub struct MediumAgent {
    rng: StdRng,
}

impl MediumAgent {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn choose_discard(rng: &mut impl Rng, view: &PlayerView, legal: &[Action]) -> Action {
        let Some(hand) = hand_from_view(view) else {
            return legal[0];
        };

        let mut candidates: Vec<Action> = legal
            .iter()
            .copied()
            .filter(|action| matches!(action, Action::Discard(_)))
            .collect();

        if candidates.is_empty() {
            candidates = legal
                .iter()
                .copied()
                .filter(|action| matches!(action, Action::Riichi { .. }))
                .collect();
        }

        if candidates.is_empty() {
            return *legal
                .choose(rng)
                .expect("legal discard actions expected in discard phase");
        }

        let mut best_waiting = 0usize;
        let mut best: Vec<Action> = Vec::new();
        for action in candidates {
            let tile = match action {
                Action::Discard(tile) | Action::Riichi { discard: tile } => tile,
                _ => continue,
            };
            let Some(after) = hand_without_concealed_tile(&hand, tile) else {
                continue;
            };
            let value = waiting_count(&after);
            if value > best_waiting {
                best_waiting = value;
                best.clear();
                best.push(action);
            } else if value == best_waiting {
                best.push(action);
            }
        }

        *best
            .choose(rng)
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

        let Some(pending) = view.turn.pending_call() else {
            return Action::Pass;
        };

        let mut best = Action::Pass;
        let mut best_waiting = baseline;

        for action in legal {
            if matches!(action, Action::Pass | Action::Ron) {
                continue;
            }
            let Some(after) = simulate_call(&hand, *action, pending.tile) else {
                continue;
            };
            let value = best_waiting_potential(&after);
            if value > best_waiting {
                best_waiting = value;
                best = *action;
            }
        }

        if best_waiting > baseline {
            best
        } else {
            Action::Pass
        }
    }
}

impl Agent for MediumAgent {
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action {
        if legal.is_empty() {
            panic!("MediumAgent asked to decide with no legal actions");
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
