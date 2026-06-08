mod snapshot;

#[cfg(test)]
mod tests;

pub use snapshot::MatchSnapshot;

use crate::Error;
use crate::event::Event;
use crate::game::Match;
use crate::rules::{RulesConfig, RulesProfileId};
use crate::state::HandPhase;

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
        let mut hand_starts = 0usize;

        let mut index = 0usize;
        while index < self.events.len() {
            let event = &self.events[index];
            let next = self.events.get(index + 1);

            match event {
                Event::HandStarted {
                    dealer,
                    round_wind,
                    kyoku,
                    honba,
                } => {
                    hand_starts += 1;
                    if hand_starts > 1 {
                        game.begin_hand_from_event(*dealer, *round_wind, *kyoku, *honba)?;
                    } else {
                        game.assert_hand_metadata(*dealer, *round_wind, *kyoku, *honba)?;
                    }
                }
                Event::Dealt { dealer } => {
                    if hand_starts > 1 && game.dealer() != *dealer {
                        return Err(Error::ReplayMismatch {
                            detail: "dealt dealer mismatch on new hand",
                        });
                    }
                }
                Event::MatchEnded { scores } => {
                    game.end_with_scores(*scores);
                }
                Event::Discarded { seat, .. } => {
                    game.hand_mut().apply_event(event)?;
                    if game.hand().phase() != HandPhase::Ended {
                        game.hand_mut().apply_discard_followup(*seat)?;
                        if game.hand().phase() == HandPhase::Reaction
                            && matches!(next, Some(Event::Drawn { .. }))
                        {
                            game.hand_mut().resolve_all_passed_reaction()?;
                        }
                    }
                    game.sync_scores_from_hand();
                }
                _ => {
                    game.hand_mut().apply_event(event)?;
                    game.sync_scores_from_hand();
                }
            }

            index += 1;
        }

        Ok(game)
    }

    pub fn snapshots(&self) -> Result<Vec<MatchSnapshot>, Error> {
        let mut game = Match::new(self.rules_config.clone(), self.seed)?;
        let mut out = vec![game.snapshot()];

        let mut hand_starts = 0usize;
        let mut index = 0usize;
        while index < self.events.len() {
            let event = &self.events[index];
            let next = self.events.get(index + 1);

            match event {
                Event::HandStarted {
                    dealer,
                    round_wind,
                    kyoku,
                    honba,
                } => {
                    hand_starts += 1;
                    if hand_starts > 1 {
                        game.begin_hand_from_event(*dealer, *round_wind, *kyoku, *honba)?;
                    } else {
                        game.assert_hand_metadata(*dealer, *round_wind, *kyoku, *honba)?;
                    }
                }
                Event::Dealt { .. } => {}
                Event::MatchEnded { scores } => game.end_with_scores(*scores),
                Event::Discarded { seat, .. } => {
                    game.hand_mut().apply_event(event)?;
                    if game.hand().phase() != HandPhase::Ended {
                        game.hand_mut().apply_discard_followup(*seat)?;
                        if game.hand().phase() == HandPhase::Reaction
                            && matches!(next, Some(Event::Drawn { .. }))
                        {
                            game.hand_mut().resolve_all_passed_reaction()?;
                        }
                    }
                    game.sync_scores_from_hand();
                }
                _ => {
                    game.hand_mut().apply_event(event)?;
                    game.sync_scores_from_hand();
                }
            }

            out.push(game.snapshot());
            index += 1;
        }

        Ok(out)
    }
}
