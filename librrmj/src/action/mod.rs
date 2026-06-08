use crate::tile::Tile;

/// Player intent submitted to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Draw one tile from the live wall (after the previous discard).
    Draw,
    /// Discard a tile from the concealed hand.
    Discard(Tile),
    /// Decline to call on another player's discard.
    Pass,
    /// Complete a chii using the last discard and two concealed tiles.
    Chi { tiles: [Tile; 3] },
    /// Pon the last discard using two matching concealed tiles.
    Pon,
    /// Open kan (daiminkan) on the last discard using three matching concealed tiles.
    OpenKan,
    /// Closed kan (ankan) on the current turn using four matching concealed tiles.
    ClosedKan { tile: Tile },
}
