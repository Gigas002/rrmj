#[cfg(test)]
mod tests;

use crate::tile::Tile;

/// One grouped set in a hand path (sequence, triplet, pair, or open meld).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathGroup {
    pub tiles: Vec<Tile>,
    /// Open meld on the table (chi / pon / kan).
    pub open: bool,
}

/// Concrete hand shape for a recommendation row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDecomposition {
    pub groups: Vec<PathGroup>,
    /// Tile(s) that complete this path from the current visible hand.
    pub missing: Vec<Tile>,
    /// Discard that reaches tenpai (1-shanten rows only).
    pub suggested_discard: Option<Tile>,
}

impl PathDecomposition {
    pub const fn empty() -> Self {
        Self {
            groups: Vec::new(),
            missing: Vec::new(),
            suggested_discard: None,
        }
    }

    pub fn format_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(discard) = self.suggested_discard {
            lines.push(format!("Discard {discard} →"));
        }
        if !self.groups.is_empty() {
            lines.push(
                self.groups
                    .iter()
                    .map(format_group)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        if !self.missing.is_empty() {
            let waits: String = self
                .missing
                .iter()
                .map(|tile| format!("+{tile}"))
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(format!("Need {waits}"));
        }
        lines
    }
}

fn format_group(group: &PathGroup) -> String {
    let mut tiles = group.tiles.clone();
    tiles.sort_by(|a, b| a.cmp_sort(*b));
    let body: String = tiles.iter().map(|t| t.to_string()).collect();
    if group.open {
        format!("[{body}]")
    } else {
        body
    }
}
