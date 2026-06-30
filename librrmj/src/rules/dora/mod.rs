//! Dora matching helpers for clients (no scoring logic).

#[cfg(test)]
mod tests;

use crate::tile::Tile;

pub use crate::rules::standard::dora::dora_tile;

/// Whether `tile` matches any current dora indicator.
pub fn matches_indicator_dora(tile: Tile, indicators: &[Tile]) -> bool {
    indicators
        .iter()
        .filter_map(|&indicator| dora_tile(indicator))
        .any(|dora| tile.matches_identity(dora))
}

/// Whether `tile` counts as aka dora when the ruleset enables it.
pub fn is_aka_dora(tile: Tile, aka_enabled: bool) -> bool {
    aka_enabled && tile.is_red()
}

/// Indicator dora or aka dora (when enabled).
pub fn is_hand_dora(tile: Tile, indicators: &[Tile], aka_enabled: bool) -> bool {
    matches_indicator_dora(tile, indicators) || is_aka_dora(tile, aka_enabled)
}
