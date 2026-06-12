use crate::game::Game;
use crate::game::RoundWind;
use crate::hand::Meld;
use crate::state::{HandPhase, HandState, SEAT_COUNT};
use crate::tile::Tile;

/// A discard other seats may react to during [`HandPhase::Reaction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingCall {
    pub discarder: usize,
    pub tile: Tile,
}

/// What this seat's turn is focused on right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnFocus {
    /// Waiting for draw or between actions.
    Idle,
    /// A tile is callable (discard or kakan) during reaction.
    Reaction,
    /// This seat is choosing a discard / kan / riichi / tsumo.
    Discarding { drawn: Option<Tile> },
}

/// Table turn state visible to one seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnContext {
    /// Tile to highlight in a river, or to call during [`TurnFocus::Reaction`].
    pub latest_discard: Option<PendingCall>,
    pub focus: TurnFocus,
}

impl TurnContext {
    pub const fn idle() -> Self {
        Self {
            latest_discard: None,
            focus: TurnFocus::Idle,
        }
    }

    pub fn reaction(call: PendingCall) -> Self {
        Self {
            latest_discard: Some(call),
            focus: TurnFocus::Reaction,
        }
    }

    pub fn discarding(drawn: Option<Tile>) -> Self {
        Self {
            latest_discard: None,
            focus: TurnFocus::Discarding { drawn },
        }
    }

    /// Callable tile during an active reaction window.
    pub fn pending_call(&self) -> Option<PendingCall> {
        matches!(self.focus, TurnFocus::Reaction)
            .then(|| self.latest_discard)
            .flatten()
    }

    /// Tile just drawn on this seat's discard turn (hidden from other seats).
    pub fn drawn_tile(&self) -> Option<Tile> {
        match self.focus {
            TurnFocus::Discarding { drawn } => drawn,
            _ => None,
        }
    }
}

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
    pub turn: TurnContext,
}

impl PlayerView {
    pub fn from_game(game: &Game, seat: usize) -> Self {
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
            turn: turn_context_from_hand(hand, seat),
        }
    }
}

fn turn_context_from_hand(hand: &HandState, seat: usize) -> TurnContext {
    let latest_discard = if hand.phase() == HandPhase::Reaction {
        hand.pending_call()
    } else {
        hand.last_discard()
    };

    let focus = match hand.phase() {
        HandPhase::Reaction => TurnFocus::Reaction,
        HandPhase::Discard if seat == hand.current_actor() => TurnFocus::Discarding {
            drawn: hand.last_drawn_tile(),
        },
        _ => TurnFocus::Idle,
    };

    TurnContext {
        latest_discard,
        focus,
    }
}
