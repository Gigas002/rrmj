use crate::game::{AbortiveDrawKind, RoundWind};
use crate::hand::MeldKind;
use crate::tile::Tile;

/// A state change that has been applied to the hand.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Event {
    /// Initial deal completed; dealer holds 14 tiles, others hold 13.
    Dealt { dealer: usize },
    /// A seat drew a tile from the live wall.
    Drawn { seat: usize, tile: Tile },
    /// A seat discarded a tile to the river.
    Discarded { seat: usize, tile: Tile },
    /// A seat declared riichi with a discard.
    RiichiDeclared { seat: usize, discard: Tile },
    /// A seat called the last discard.
    Called {
        seat: usize,
        from: usize,
        meld: MeldKind,
        tiles: Vec<Tile>,
    },
    /// A new dora indicator was revealed after a kan.
    DoraRevealed { tile: Tile },
    /// A rinshan draw after a kan.
    RinshanDrawn { seat: usize, tile: Tile },
    /// A seat upgraded an open pon; tile is exposed for chankan until the reaction resolves.
    KakanDeclared {
        seat: usize,
        meld_index: usize,
        tile: Tile,
    },
    /// A seat won the hand.
    Won { seat: usize, han: u8, fu: u8 },
    /// Score transfers applied to all seats.
    ScoresAdjusted { deltas: [i32; 4] },
    /// Live wall exhausted without a win.
    ExhaustiveDraw { deltas: [i32; 4] },
    /// A new hand began within the match.
    HandStarted {
        dealer: usize,
        round_wind: RoundWind,
        kyoku: u8,
        honba: u8,
    },
    /// Hand ended in an abortive draw.
    AbortiveDraw { kind: AbortiveDrawKind },
    /// Match completed.
    MatchEnded { scores: [i32; 4] },
}
