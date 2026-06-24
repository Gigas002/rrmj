/// Table wind for the current round (East or South half).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoundWind {
    East,
    South,
}

impl RoundWind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::East => "east",
            Self::South => "south",
        }
    }
}

/// How many round winds to play before the game session ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameLength {
    /// East 1–4 only.
    EastOnly,
    /// East 1–4 then South 1–4 (standard hanchan).
    Hanchan,
}

/// Reason an in-progress hand ended in an abortive draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbortiveDrawKind {
    NineTerminals,
    FourWinds,
    FourKongs,
    FourRiichis,
}

/// Phase of a multi-hand game session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamePhase {
    InHand,
    Ended,
}

/// Outcome of a single hand for match-flow bookkeeping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandOutcome {
    Win { winners: Vec<usize> },
    ExhaustiveDraw,
    AbortiveDraw(AbortiveDrawKind),
}
