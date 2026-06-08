use thiserror::Error;

use crate::action::Action;
use crate::hand::MeldKind;
use crate::state::CallKind;
use crate::state::HandPhase;
use crate::tile::Tile;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown rules profile: {0}")]
    UnknownRulesProfile(String),

    #[error("invalid tile string: {0}")]
    InvalidTileString(String),

    #[error("invalid seat index: {0}")]
    InvalidSeat(usize),

    #[error("hand for seat {seat} has {actual} tiles, expected {expected}")]
    InvalidHandTileCount {
        seat: usize,
        expected: usize,
        actual: usize,
    },

    #[error("{kind:?} meld has {actual} tiles, expected {expected}")]
    InvalidMeldTileCount {
        kind: MeldKind,
        expected: usize,
        actual: usize,
    },

    #[error("{kind:?} meld requires a called tile")]
    MissingCalledTile { kind: MeldKind },

    #[error("{kind:?} meld must not have a called tile")]
    UnexpectedCalledTile { kind: MeldKind },

    #[error("live wall is exhausted")]
    LiveWallExhausted,

    #[error("dead wall has {actual} tiles, expected {expected}")]
    InvalidDeadWallSize { expected: usize, actual: usize },

    #[error("wall has {actual} tiles, maximum is 136")]
    InvalidWallSize { actual: usize },

    #[error("tile {tile} is not in the concealed hand")]
    TileNotInHand { tile: Tile },

    #[error("hand has already ended")]
    HandEnded,

    #[error("expected seat {expected} to act, got {actual}")]
    WrongActor { expected: usize, actual: usize },

    #[error("expected phase {expected:?}, got {actual:?}")]
    WrongPhase {
        expected: HandPhase,
        actual: HandPhase,
    },

    #[error("{action:?} is illegal in phase {phase:?}")]
    IllegalAction { action: Action, phase: HandPhase },

    #[error("expected {expected} tiles in play, found {actual}")]
    TileConservation { expected: usize, actual: usize },

    #[error("invalid chii sequence: {tiles:?}")]
    InvalidChiSequence { tiles: [Tile; 3] },

    #[error("{kind:?} call invalid: {reason}")]
    InvalidCall {
        kind: CallKind,
        reason: &'static str,
    },

    #[error("seat {seat} cannot respond to this discard")]
    NotReactingSeat { seat: usize },

    #[error("seat {seat} already responded")]
    AlreadyResponded { seat: usize },

    #[error("rinshan tiles exhausted")]
    RinshanExhausted,

    #[error("no more dora indicators to reveal")]
    DoraRevealExhausted,

    #[error("seat cannot win with the current hand")]
    CannotWin,

    #[error("seat cannot declare riichi")]
    CannotDeclareRiichi,

    #[error("seat is in furiten")]
    Furiten,

    #[error("match has ended")]
    MatchEnded,

    #[error("replay does not match engine state: {detail}")]
    ReplayMismatch { detail: &'static str },
}
