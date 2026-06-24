#[cfg(test)]
mod tests;

use crate::scoring::{WinType, Yaku};
use crate::tile::Tile;

/// One scored path toward a winning hand for planning UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recommendation {
    /// −1 = winning hand now, 0 = tenpai, 1+ = tiles away from tenpai.
    pub shanten: i8,
    /// Distinct tiles that complete this path (0 when already complete).
    pub wait_count: usize,
    /// Representative completing tile when applicable.
    pub win_tile: Option<Tile>,
    pub yaku: Vec<Yaku>,
    pub han: u8,
    pub fu: u8,
    pub dora: u8,
    pub ura_dora: u8,
    pub aka_dora: u8,
    pub expected_points: i32,
    pub win_type: WinType,
}

impl Recommendation {
    pub fn shanten_label(&self) -> String {
        match self.shanten {
            -1 => "Complete".into(),
            0 => format!("Tenpai ({} waits)", self.wait_count),
            1 => "1-shanten".into(),
            n => format!("{n}-shanten"),
        }
    }

    pub fn win_type_label(&self) -> String {
        match self.win_type {
            WinType::Tsumo => "Tsumo".into(),
            WinType::Ron { from } => format!("Ron (seat {})", from + 1),
        }
    }

    pub fn summary_line(&self) -> String {
        let yaku = self
            .yaku
            .iter()
            .map(|y| y.label())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{} — {} han / {} fu — +{}",
            yaku, self.han, self.fu, self.expected_points
        )
    }
}

/// Sort by expected score, then closeness (shanten, wait count).
pub fn sort_recommendations(paths: &mut [Recommendation]) {
    paths.sort_by(|a, b| {
        b.expected_points
            .cmp(&a.expected_points)
            .then_with(|| a.shanten.cmp(&b.shanten))
            .then_with(|| a.wait_count.cmp(&b.wait_count))
    });
}
