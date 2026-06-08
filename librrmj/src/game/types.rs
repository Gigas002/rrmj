/// Table wind for the current round (East or South half).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// How many round winds to play before the match ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchLength {
    /// East 1–4 only.
    EastOnly,
    /// East 1–4 then South 1–4 (standard hanchan).
    Hanchan,
}

/// Reason an in-progress hand ended in an abortive draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbortiveDrawKind {
    NineTerminals,
    FourWinds,
    FourKongs,
    FourRiichis,
}

/// Phase of a multi-hand match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchPhase {
    InHand,
    Ended,
}

/// Outcome of a single hand for match-flow bookkeeping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandOutcome {
    Win { winner: usize },
    ExhaustiveDraw,
    AbortiveDraw(AbortiveDrawKind),
}
