use crate::tile::Tile;

/// A state change that has been applied to the hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Initial deal completed; dealer holds 14 tiles, others hold 13.
    Dealt { dealer: usize },
    /// A seat drew a tile from the live wall.
    Drawn { seat: usize, tile: Tile },
    /// A seat discarded a tile to the river.
    Discarded { seat: usize, tile: Tile },
    /// Live wall exhausted; hand ends without calls or scoring (phase 2).
    HandEnded,
}
