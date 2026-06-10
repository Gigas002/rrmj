mod apply;
#[cfg(feature = "serde")]
mod hand_snapshot;
mod snapshot;

#[cfg(feature = "serde")]
mod recording;

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "serde"))]
mod scenario_fixtures;

pub use snapshot::MatchSnapshot;

#[cfg(feature = "serde")]
pub use hand_snapshot::HandSnapshot;
#[cfg(feature = "serde")]
pub use recording::{FORMAT_VERSION, MatchRecording, MatchStatus, PlayerSetup, RecordingMeta};

use crate::Error;
use crate::event::Event;
use crate::game::Match;
use crate::rules::{RulesConfig, RulesProfileId};

use apply::apply_events;

/// In-memory match history for regression and future file export.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Replay {
    pub rules_profile: RulesProfileId,
    pub rules_config: RulesConfig,
    pub seed: u64,
    pub events: Vec<Event>,
}

impl Replay {
    pub fn new(
        rules_profile: RulesProfileId,
        rules_config: RulesConfig,
        seed: u64,
        events: Vec<Event>,
    ) -> Self {
        Self {
            rules_profile,
            rules_config,
            seed,
            events,
        }
    }

    pub fn from_match(game: &Match) -> Self {
        Self {
            rules_profile: game.config().profile,
            rules_config: game.config().clone(),
            seed: game.seed(),
            events: game.events().to_vec(),
        }
    }

    /// Rebuild match state by applying the recorded event log.
    pub fn apply_all(&self) -> Result<Match, Error> {
        let mut game = Match::new(self.rules_config.clone(), self.seed)?;
        apply_events(&mut game, &self.events, None)?;
        Ok(game)
    }

    pub fn snapshots(&self) -> Result<Vec<MatchSnapshot>, Error> {
        let mut game = Match::new(self.rules_config.clone(), self.seed)?;
        let mut out = vec![game.snapshot()];
        let mut hand_starts = 0usize;

        for (index, event) in self.events.iter().enumerate() {
            apply::apply_one_event(
                &mut game,
                event,
                self.events.get(index + 1),
                &mut hand_starts,
            )?;
            out.push(game.snapshot());
        }

        Ok(out)
    }
}
