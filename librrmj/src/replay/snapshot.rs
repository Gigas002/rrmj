use crate::game::{Match, MatchPhase, RoundWind};
use crate::state::HandPhase;

/// Comparable match state for replay verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchSnapshot {
    pub match_phase: MatchPhase,
    pub scores: [i32; 4],
    pub dealer: usize,
    pub round_wind: RoundWind,
    pub kyoku: u8,
    pub honba: u8,
    pub hand_phase: HandPhase,
    pub current_actor: usize,
    pub concealed_counts: [usize; 4],
    pub discard_counts: [usize; 4],
    pub meld_counts: [usize; 4],
}

impl Match {
    pub fn snapshot(&self) -> MatchSnapshot {
        let hand = self.hand();
        MatchSnapshot {
            match_phase: self.phase(),
            scores: *self.scores(),
            dealer: self.dealer(),
            round_wind: self.round_wind(),
            kyoku: self.kyoku(),
            honba: self.honba(),
            hand_phase: hand.phase(),
            current_actor: hand.current_actor(),
            concealed_counts: std::array::from_fn(|s| hand.hand(s).concealed().len()),
            discard_counts: std::array::from_fn(|s| hand.discards(s).len()),
            meld_counts: std::array::from_fn(|s| hand.hand(s).melds().len()),
        }
    }
}
