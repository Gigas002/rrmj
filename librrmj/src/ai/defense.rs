use crate::agent::PlayerView;
use crate::tile::Tile;

use super::efficiency::visible_identity_count;

/// Maximum safety score returned by [`tile_safety`].
pub const SAFETY_MAX: u16 = 100;

/// Bonus applied when a numbered tile is suji-safe against a riichi opponent's discard.
pub const SUJI_BONUS: u16 = 25;

/// Bonus when all four copies of a tile identity are already visible (genbutsu).
pub const GENBUTSU_BONUS: u16 = 40;

pub fn any_opponent_riichi(view: &PlayerView) -> bool {
    view.seats
        .iter()
        .enumerate()
        .any(|(seat, info)| seat != view.seat && info.riichi)
}

/// Higher is safer to discard under riichi pressure.
pub fn tile_safety(view: &PlayerView, tile: Tile) -> u16 {
    let identity = tile.identity();
    let visible = visible_identity_count(view, identity);
    if visible >= 4 {
        return SAFETY_MAX;
    }

    let mut score = u16::from(visible) * 10;
    if visible >= 3 {
        score = score.saturating_add(GENBUTSU_BONUS);
    }

    if let Some(suit) = tile.suit() {
        if let Some(rank) = tile.rank() {
            for (seat, info) in view.seats.iter().enumerate() {
                if seat == view.seat || !info.riichi {
                    continue;
                }
                for discard in &info.discards {
                    if discard.suit() == Some(suit)
                        && let Some(d_rank) = discard.rank()
                        && rank.abs_diff(d_rank) == 3
                    {
                        score = score.saturating_add(SUJI_BONUS);
                    }
                }
            }
        }
    } else if visible >= 2 {
        // Honor tiles get safer as more copies appear in rivers and melds.
        score = score.saturating_add(15);
    }

    score.min(SAFETY_MAX)
}
