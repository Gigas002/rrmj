use crate::game::{GameLength, HandOutcome, RoundWind};
use crate::rules::RulesConfig;

/// Game-session progression policy for a rules profile.
pub trait GameFlowPolicy: Send + Sync {
    fn is_game_over(
        &self,
        round_wind: RoundWind,
        kyoku: u8,
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
        scores: &[i32; 4],
        config: &RulesConfig,
    ) -> bool {
        if let Some(target) = config.target_score
            && scores.iter().any(|&score| score >= target)
        {
            return true;
        }

        let final_round = match config.game_length {
            GameLength::EastOnly => RoundWind::East,
            GameLength::Hanchan => RoundWind::South,
        };

        round_wind == final_round && kyoku == 4
    }
}

/// Advance dealer, honba, round wind, and kyoku after a hand ends.
pub fn advance_after_hand(
    dealer: usize,
    honba: u8,
    round_wind: RoundWind,
    kyoku: u8,
    outcome: HandOutcome,
    dealer_tenpai: bool,
) -> (usize, u8, RoundWind, u8) {
    use crate::game::AbortiveDrawKind;

    if matches!(
        outcome,
        HandOutcome::AbortiveDraw(AbortiveDrawKind::NineTerminals | AbortiveDrawKind::FourWinds)
    ) {
        return (dealer, honba, round_wind, kyoku);
    }

    let renchan = match outcome {
        HandOutcome::Win { winners } => winners.contains(&dealer),
        HandOutcome::ExhaustiveDraw => dealer_tenpai,
        HandOutcome::AbortiveDraw(AbortiveDrawKind::FourKongs | AbortiveDrawKind::FourRiichis) => {
            dealer_tenpai
        }
        HandOutcome::AbortiveDraw(_) => false,
    };

    if renchan {
        return (dealer, honba.saturating_add(1), round_wind, kyoku);
    }

    let honba = 0;
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
