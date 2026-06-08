use crate::tile::Tile;

/// When to evaluate abortive-draw rules after a state change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortiveTrigger {
    DealerFirstTurn,
    FirstDiscard { seat: usize, tile: Tile },
    KanDeclared,
    RiichiDeclared,
}
