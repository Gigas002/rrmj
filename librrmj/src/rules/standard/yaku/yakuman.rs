//! Kokushi musou tile check (shared by win-shape detection and scoring).

use crate::tile::{Dragon, Suit, Tile, TileIdentity, Wind};

pub fn is_kokushi_tiles(tiles: &[Tile]) -> bool {
    if tiles.len() != 14 {
        return false;
    }
    let mut counts = std::collections::HashMap::new();
    for tile in tiles {
        *counts.entry(tile.identity()).or_insert(0u8) += 1;
    }
    if counts.len() != 13 {
        return false;
    }
    kokushi_identities()
        .iter()
        .all(|id| counts.get(id).copied().unwrap_or(0) >= 1)
        && counts.values().sum::<u8>() == 14
}

fn kokushi_identities() -> [TileIdentity; 13] {
    [
        TileIdentity::Numbered {
            suit: Suit::Man,
            rank: 1,
        },
        TileIdentity::Numbered {
            suit: Suit::Man,
            rank: 9,
        },
        TileIdentity::Numbered {
            suit: Suit::Pin,
            rank: 1,
        },
        TileIdentity::Numbered {
            suit: Suit::Pin,
            rank: 9,
        },
        TileIdentity::Numbered {
            suit: Suit::Sou,
            rank: 1,
        },
        TileIdentity::Numbered {
            suit: Suit::Sou,
            rank: 9,
        },
        TileIdentity::Wind(Wind::East),
        TileIdentity::Wind(Wind::South),
        TileIdentity::Wind(Wind::West),
        TileIdentity::Wind(Wind::North),
        TileIdentity::Dragon(Dragon::White),
        TileIdentity::Dragon(Dragon::Green),
        TileIdentity::Dragon(Dragon::Red),
    ]
}
