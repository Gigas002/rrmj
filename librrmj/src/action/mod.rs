use crate::tile::Tile;

/// Player intent submitted to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    /// Draw one tile from the live wall (after the previous discard).
    Draw,
    /// Discard a tile from the concealed hand.
    Discard(Tile),
    /// Declare riichi and discard the chosen tile.
    Riichi { discard: Tile },
    /// Win on a self-draw.
    Tsumo,
    /// Win on another player's discard.
    Ron,
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
    /// Upgrade an open pon to kan using the fourth matching tile from hand.
    Kakan { meld_index: usize },
    /// Dealer aborts on the first turn with nine or more terminal/honor types.
    AbortiveNineTerminals,
}
