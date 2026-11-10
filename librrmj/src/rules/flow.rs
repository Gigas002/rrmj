use crate::game::{GameLength, HandOutcome, RoundWind};
use crate::rules::RulesConfig;

/// Game-session progression policy for a rules profile.
pub trait GameFlowPolicy: Send + Sync {
    /// Whether the match ends after the hand just played (`round_wind` /
    /// `kyoku` / `dealer` describe that hand; `renchan` is whether the dealer
    /// would keep the seat).
    fn is_game_over(
        &self,
        round_wind: RoundWind,
        kyoku: u8,
        dealer: usize,
        renchan: bool,
        scores: &[i32; 4],
        config: &RulesConfig,
    ) -> bool;
}

pub struct StandardGameFlow;

impl GameFlowPolicy for StandardGameFlow {
    fn is_game_over(
        &self,
        round_wind: RoundWind,
        kyoku: u8,
        dealer: usize,
        renchan: bool,
        scores: &[i32; 4],
        config: &RulesConfig,
    ) -> bool {
        // Bust (tobi): a seat below zero ends the match.
        if scores.iter().any(|&score| score < 0) {
            return true;
        }
        if let Some(target) = config.target_score
            && scores.iter().any(|&score| score >= target)
        {
            return true;
        }

        let final_round = match config.game_length {
            GameLength::EastOnly => RoundWind::East,
            GameLength::Hanchan => RoundWind::South,
        };
        if round_wind != final_round || kyoku != 4 {
            return false;
        }
        // All-last: a dealer repeat plays on unless the dealer already leads
        // (agari-yame / tenpai-yame).
        !renchan || placement_order(scores)[0] == dealer
    }
}

/// Seats from first to last place; equal scores rank by seat order (seat 0
/// was the first dealer).
pub fn placement_order(scores: &[i32; 4]) -> [usize; 4] {
    let mut seats = [0, 1, 2, 3];
    seats.sort_by(|&a, &b| scores[b].cmp(&scores[a]).then(a.cmp(&b)));
    seats
}

/// Advance dealer, honba, round wind, and kyoku after a hand ends.
///
/// Honba goes up on a dealer win and on every draw; only a non-dealer win
/// resets it. Abortive draws always keep the dealer.
pub fn advance_after_hand(
    dealer: usize,
    honba: u8,
    round_wind: RoundWind,
    kyoku: u8,
    outcome: HandOutcome,
    dealer_tenpai: bool,
) -> (usize, u8, RoundWind, u8) {
    use crate::game::AbortiveDrawKind;

    let (renchan, next_honba) = match &outcome {
        HandOutcome::Win { winners } => {
            let renchan = winners.contains(&dealer);
            (renchan, if renchan { honba.saturating_add(1) } else { 0 })
        }
        HandOutcome::ExhaustiveDraw => (dealer_tenpai, honba.saturating_add(1)),
        HandOutcome::AbortiveDraw(
            AbortiveDrawKind::NineTerminals
            | AbortiveDrawKind::FourWinds
            | AbortiveDrawKind::FourKongs
            | AbortiveDrawKind::FourRiichis,
        ) => (true, honba.saturating_add(1)),
    };

    if renchan {
        return (dealer, next_honba, round_wind, kyoku);
    }

    let honba = next_honba;
    let dealer = (dealer + 1) % 4;

    if dealer == 0 {
        let round_wind = match round_wind {
            RoundWind::East => RoundWind::South,
            RoundWind::South => RoundWind::South,
        };
        (dealer, honba, round_wind, 1)
    } else {
        (dealer, honba, round_wind, kyoku.saturating_add(1))
    }
}
