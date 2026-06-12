use crate::Error;
use crate::game::{Game, GamePhase, RoundWind};
use crate::hand::Hand;
use crate::rules::RulesConfig;
use crate::state::{HandEndReason, HandPhase, ReactionState};
use crate::tile::Tile;
use crate::wall::WallSnapshot;

/// Full in-hand tile and flow state at a save point.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandSnapshot {
    pub dealer: usize,
    pub current_actor: usize,
    pub phase: HandPhase,
    pub hands: [Hand; 4],
    pub discards: [Vec<Tile>; 4],
    pub wall: WallSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction: Option<ReactionState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_discard: Option<(usize, Tile)>,
    pub scores: [i32; 4],
    pub riichi: [bool; 4],
    pub table_riichi_sticks: u8,
    pub honba: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_draw: Option<Tile>,
    pub first_discards: [Option<Tile>; 4],
    pub is_dealer_first_turn: bool,
    #[serde(default)]
    pub temporary_furiten: [bool; 4],
    #[serde(default)]
    pub riichi_furiten: [bool; 4],
    #[serde(default)]
    pub ippatsu_live: [bool; 4],
    #[serde(default)]
    pub double_riichi: [bool; 4],
    #[serde(default)]
    pub calls_made: bool,
    #[serde(default)]
    pub is_rinshan_draw: bool,
    #[serde(default)]
    pub live_draws: [u8; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_reason: Option<HandEndReason>,
}

impl HandSnapshot {
    pub fn restore(&self, config: RulesConfig) -> Result<crate::state::HandState, Error> {
        crate::state::HandState::from_snapshot(self.clone(), config)
    }
}

/// Comparable game state for replay verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSnapshot {
    pub game_phase: GamePhase,
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

impl Game {
    pub fn snapshot(&self) -> GameSnapshot {
        let hand = self.hand();
        GameSnapshot {
            game_phase: self.phase(),
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
