use crate::tile::Tile;

/// Player intent submitted to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Draw one tile from the live wall (after the previous discard).
    Draw,
    /// Discard a tile from the concealed hand.
    Discard(Tile),
    /// Decline to call (used in later reaction phases).
    Pass,
}
