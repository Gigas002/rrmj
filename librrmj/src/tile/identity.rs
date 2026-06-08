use super::{Dragon, Suit, Tile, TileKind, Wind};

/// Tile identity for calls (red fives match normal fives of the same suit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileIdentity {
    Numbered { suit: Suit, rank: u8 },
    Wind(Wind),
    Dragon(Dragon),
}

impl Tile {
    pub fn identity(self) -> TileIdentity {
        match self.kind {
            TileKind::Man(r) => TileIdentity::Numbered {
                suit: Suit::Man,
                rank: r,
            },
            TileKind::Pin(r) => TileIdentity::Numbered {
                suit: Suit::Pin,
                rank: r,
            },
            TileKind::Sou(r) => TileIdentity::Numbered {
                suit: Suit::Sou,
                rank: r,
            },
            TileKind::Wind(w) => TileIdentity::Wind(w),
            TileKind::Dragon(d) => TileIdentity::Dragon(d),
        }
    }

    pub fn matches_identity(self, other: Tile) -> bool {
        self.identity() == other.identity()
    }
}
