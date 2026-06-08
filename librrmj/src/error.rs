use thiserror::Error;

use crate::hand::MeldKind;

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
}
