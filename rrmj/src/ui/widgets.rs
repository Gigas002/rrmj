use librrmj::hand::Meld;
use librrmj::tile::Tile;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme::Theme;
use crate::ui::render::{meld_kind_label, tile_label};

#[cfg(test)]
mod tests;

/// Whether `candidate` should receive match emphasis for `reference` (aka ↔ normal five).
pub(crate) fn tile_matches_highlight(reference: Tile, candidate: Tile) -> bool {
    reference.matches_identity(candidate)
}

pub fn tile_span(
    tile: Tile,
    theme: &Theme,
    selected: bool,
    drawn: bool,
    matched: bool,
    recent_discard: bool,
) -> Span<'static> {
    Span::styled(
        tile_label(tile),
        theme.tile_style(tile.is_red(), selected, drawn, matched, recent_discard),
    )
}

pub fn tiles_line(
    tiles: &[Tile],
    theme: &Theme,
    selected: Option<usize>,
    drawn: Option<usize>,
    match_tile: Option<Tile>,
    recent_index: Option<usize>,
) -> Line<'static> {
    Line::from(
        tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                tile_span(
                    *t,
                    theme,
                    selected == Some(i),
                    drawn == Some(i),
                    match_tile.is_some_and(|m| tile_matches_highlight(m, *t)),
                    recent_index == Some(i),
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub fn meld_line(meld: &Meld, theme: &Theme) -> Line<'static> {
    let kind = meld_kind_label(meld.kind());
    let tiles: String = meld.tiles().iter().map(|t| tile_label(*t)).collect();
    Line::from(Span::styled(
        format!("{kind}: {tiles}"),
        Style::default().fg(theme.primary),
    ))
}

pub fn riichi_badge(theme: &Theme, pulsing: bool) -> Span<'static> {
    Span::styled(" RIICHI", theme.riichi_style(pulsing))
}

pub fn muted_span(text: impl Into<String>, theme: &Theme) -> Span<'static> {
    Span::styled(
        text.into(),
        Style::default().fg(theme.muted).add_modifier(Modifier::DIM),
    )
}
