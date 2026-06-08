mod trigger;
mod types;

#[cfg(test)]
mod tests;

pub use trigger::AbortiveTrigger;
pub use types::{AbortiveDrawKind, HandOutcome, MatchLength, MatchPhase, RoundWind};

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::Error;
use crate::action::Action;
use crate::event::Event;
use crate::rules::flow::advance_after_hand;
use crate::rules::{RulesConfig, RulesRegistry};
use crate::state::{HandEndReason, HandState};
use crate::wall::Wall;

/// Multi-hand match with round progression, honba, and renchan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    config: RulesConfig,
    seed: u64,
    dealer: usize,
    round_wind: RoundWind,
    kyoku: u8,
    honba: u8,
    scores: [i32; 4],
    table_riichi_sticks: u8,
    hand: HandState,
    phase: MatchPhase,
    hand_index: u32,
}

impl Match {
    pub fn new(config: RulesConfig, seed: u64) -> Result<Self, Error> {
        let scores = [config.starting_points; 4];
        let mut wall = Wall::new(&config, hand_rng(seed, 0));
        let deal = wall.deal(0)?;
        let hand = HandState::from_deal_with_carry(wall, deal, config.clone(), scores, 0, 0);

        Ok(Self {
            config,
            seed,
            dealer: 0,
            round_wind: RoundWind::East,
            kyoku: 1,
            honba: 0,
            scores,
            table_riichi_sticks: 0,
            hand,
            phase: MatchPhase::InHand,
            hand_index: 0,
        })
    }

    pub const fn config(&self) -> &RulesConfig {
        &self.config
    }

    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub const fn dealer(&self) -> usize {
        self.dealer
    }

    pub const fn round_wind(&self) -> RoundWind {
        self.round_wind
    }

    pub const fn kyoku(&self) -> u8 {
        self.kyoku
    }

    pub const fn honba(&self) -> u8 {
        self.honba
    }

    pub fn scores(&self) -> &[i32; 4] {
        &self.scores
    }

    pub const fn phase(&self) -> MatchPhase {
        self.phase
    }

    pub fn hand(&self) -> &HandState {
        &self.hand
    }

    pub fn is_ended(&self) -> bool {
        self.phase == MatchPhase::Ended
    }

    /// Events emitted when the first hand is dealt (for callers that need the log).
    pub fn start_events(&self) -> Vec<Event> {
        vec![
            Event::Dealt {
                dealer: self.dealer,
            },
            Event::HandStarted {
                dealer: self.dealer,
                round_wind: self.round_wind,
                kyoku: self.kyoku,
                honba: self.honba,
            },
        ]
    }

    pub fn apply_action(&mut self, seat: usize, action: Action) -> Result<Vec<Event>, Error> {
        if self.phase == MatchPhase::Ended {
            return Err(Error::MatchEnded);
        }

        let mut events = self.hand.apply(seat, action)?;
        if self.hand.is_ended() {
            events.extend(self.finish_hand()?);
        }
        Ok(events)
    }

    fn finish_hand(&mut self) -> Result<Vec<Event>, Error> {
        self.scores = *self.hand.scores();
        let outcome = hand_outcome(self.hand.end_reason());
        let profile = RulesRegistry::get(self.config.profile)?;
        let dealer_tenpai = profile.is_tenpai(self.hand.hand(self.dealer), &self.config);

        if profile.match_flow().is_match_over(
            self.round_wind,
            self.kyoku,
            &self.scores,
            &self.config,
        ) {
            self.table_riichi_sticks = 0;
            self.phase = MatchPhase::Ended;
            return Ok(vec![Event::MatchEnded {
                scores: self.scores,
            }]);
        }

        let riichi_sticks = self.hand.table_riichi_sticks();
        self.table_riichi_sticks = match outcome {
            HandOutcome::Win { .. } => 0,
            _ => riichi_sticks,
        };

        let (dealer, honba, round_wind, kyoku) = advance_after_hand(
            self.dealer,
            self.honba,
            self.round_wind,
            self.kyoku,
            outcome,
            dealer_tenpai,
        );

        self.dealer = dealer;
        self.honba = honba;
        self.round_wind = round_wind;
        self.kyoku = kyoku;
        self.hand_index += 1;

        let (hand, start_events) = self.deal_hand()?;
        self.hand = hand;
        self.phase = MatchPhase::InHand;
        Ok(start_events)
    }

    #[cfg(test)]
    pub(crate) fn hand_mut(&mut self) -> &mut HandState {
        &mut self.hand
    }

    #[cfg(test)]
    pub(crate) fn finish_hand_for_test(&mut self) -> Result<Vec<Event>, Error> {
        self.finish_hand()
    }

    fn deal_hand(&self) -> Result<(HandState, Vec<Event>), Error> {
        let mut wall = Wall::new(&self.config, hand_rng(self.seed, self.hand_index));
        let deal = wall.deal(self.dealer)?;
        let hand = HandState::from_deal_with_carry(
            wall,
            deal,
            self.config.clone(),
            self.scores,
            self.honba,
            self.table_riichi_sticks,
        );
        let events = vec![
            Event::Dealt {
                dealer: self.dealer,
            },
            Event::HandStarted {
                dealer: self.dealer,
                round_wind: self.round_wind,
                kyoku: self.kyoku,
                honba: self.honba,
            },
        ];
        Ok((hand, events))
    }
}

fn hand_rng(seed: u64, hand_index: u32) -> StdRng {
    StdRng::seed_from_u64(seed.wrapping_add(hand_index as u64))
}

fn hand_outcome(reason: Option<HandEndReason>) -> HandOutcome {
    match reason {
        Some(HandEndReason::Win { winner }) => HandOutcome::Win { winner },
        Some(HandEndReason::ExhaustiveDraw) => HandOutcome::ExhaustiveDraw,
        Some(HandEndReason::AbortiveDraw(kind)) => HandOutcome::AbortiveDraw(kind),
        None => HandOutcome::ExhaustiveDraw,
    }
}
