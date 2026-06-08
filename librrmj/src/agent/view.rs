use crate::game::Match;
use crate::game::RoundWind;
use crate::hand::Meld;
use crate::state::{HandPhase, HandState, SEAT_COUNT};
use crate::tile::Tile;

/// Public information about one seat as visible from the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatView {
    pub melds: Vec<Meld>,
    pub discards: Vec<Tile>,
    pub riichi: bool,
    pub concealed_count: usize,
}

/// Information available to one seat (concealed tiles only for self).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerView {
    pub seat: usize,
    pub dealer: usize,
    pub phase: HandPhase,
    pub current_actor: usize,
    pub round_wind: RoundWind,
    pub kyoku: u8,
    pub honba: u8,
    pub scores: [i32; 4],
    pub own_concealed: Vec<Tile>,
    pub seats: [SeatView; SEAT_COUNT],
    pub dora_indicators: Vec<Tile>,
    pub table_riichi_sticks: u8,
}

impl PlayerView {
    pub fn from_match(game: &Match, seat: usize) -> Self {
        Self::from_hand(
            game.hand(),
            seat,
            game.dealer(),
            game.round_wind(),
            game.kyoku(),
            game.honba(),
            *game.scores(),
        )
    }

    pub fn from_hand(
        hand: &HandState,
        seat: usize,
        dealer: usize,
        round_wind: RoundWind,
        kyoku: u8,
        honba: u8,
        scores: [i32; 4],
    ) -> Self {
        let mut seats = std::array::from_fn(|_| SeatView {
            melds: Vec::new(),
            discards: Vec::new(),
            riichi: false,
            concealed_count: 0,
        });

        for (s, seat_view) in seats.iter_mut().enumerate() {
            *seat_view = SeatView {
                melds: hand.hand(s).melds().to_vec(),
                discards: hand.discards(s).to_vec(),
                riichi: hand.is_riichi(s),
                concealed_count: hand.hand(s).concealed().len(),
            };
        }

        Self {
            seat,
            dealer,
            phase: hand.phase(),
            current_actor: hand.current_actor(),
            round_wind,
            kyoku,
            honba,
            scores,
            own_concealed: hand.hand(seat).concealed().tiles().to_vec(),
            seats,
            dora_indicators: hand.wall().dora_indicators(),
            table_riichi_sticks: hand.table_riichi_sticks(),
        }
    }
}
