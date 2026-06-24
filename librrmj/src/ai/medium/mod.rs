use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::action::Action;
use crate::agent::{Agent, PlayerView};
use crate::state::HandPhase;

use super::common::prefer_win;
use super::strategy::{CallPolicy, DiscardPolicy, choose_discard, choose_reaction};

/// Shanten-guided discards, conservative calls, basic kan when clearly beneficial.
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
            HandPhase::Reaction => choose_reaction(view, legal, CallPolicy::Conservative),
            HandPhase::Discard => choose_discard(&mut self.rng, view, legal, DiscardPolicy::MEDIUM),
            _ => *legal
                .choose(&mut self.rng)
                .expect("legal actions must be non-empty"),
        }
    }
}
