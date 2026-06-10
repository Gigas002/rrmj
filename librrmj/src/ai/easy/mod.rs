use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::action::Action;
use crate::agent::{Agent, PlayerView};

/// Picks a random legal action, always taking an obvious win when offered.
#[derive(Debug)]
pub struct EasyAgent {
    rng: StdRng,
}

impl EasyAgent {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn decide_with_rng(rng: &mut impl Rng, legal: &[Action]) -> Action {
        for &action in legal {
            if matches!(action, Action::Tsumo | Action::Ron) {
                return action;
            }
        }
        *legal
            .choose(rng)
            .expect("legal actions must be non-empty for EasyAgent")
    }
}

impl Agent for EasyAgent {
    fn decide(&mut self, _: &PlayerView, legal: &[Action]) -> Action {
        Self::decide_with_rng(&mut self.rng, legal)
    }
}
