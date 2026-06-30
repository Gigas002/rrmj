use librrmj::hand::Meld;
use librrmj::rules::is_hand_dora;
use librrmj::tile::Tile;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme::Theme;
use crate::ui::render::{meld_kind_label, tile_label};

#[cfg(test)]
mod tests;

/// Per-tile emphasis flags for hand, meld, and river rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct TileHighlight<'a> {
    pub selected: bool,
    pub drawn: bool,
    pub match_tile: Option<Tile>,
    pub recent_discard: bool,
    pub dora_indicators: &'a [Tile],
    pub aka_dora: bool,
}

/// Whether `candidate` should receive match emphasis for `reference` (aka ↔ normal five).
pub(crate) fn tile_matches_highlight(reference: Tile, candidate: Tile) -> bool {
    reference.matches_identity(candidate)
}

pub fn tile_span(tile: Tile, theme: &Theme, highlight: TileHighlight<'_>) -> Span<'static> {
    let matched = highlight
        .match_tile
        .is_some_and(|m| tile_matches_highlight(m, tile));
    let is_dora = is_hand_dora(tile, highlight.dora_indicators, highlight.aka_dora);
    Span::styled(
        tile_label(tile),
        theme.tile_style(
            tile.is_red(),
            highlight.selected,
            highlight.drawn,
            matched,
            highlight.recent_discard,
            is_dora,
        ),
    )
}

pub fn tiles_line(tiles: &[Tile], theme: &Theme, ctx: TilesLineContext<'_>) -> Line<'static> {
    Line::from(
        tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                tile_span(
                    *t,
                    theme,
                    TileHighlight {
                        selected: ctx.selected == Some(i),
                        drawn: ctx.drawn == Some(i),
                        match_tile: ctx.match_tile,
                        recent_discard: ctx.recent_index == Some(i),
                        dora_indicators: ctx.dora_indicators,
                        aka_dora: ctx.aka_dora,
                    },
                )
            })
            .collect::<Vec<_>>(),
    )
}

/// River / hand line context for [`tiles_line`].
#[derive(Debug, Clone, Copy, Default)]
pub struct TilesLineContext<'a> {
    pub selected: Option<usize>,
    pub drawn: Option<usize>,
    pub match_tile: Option<Tile>,
    pub recent_index: Option<usize>,
    pub dora_indicators: &'a [Tile],
    pub aka_dora: bool,
}

impl<'a> TilesLineContext<'a> {
    pub const fn empty() -> Self {
        Self {
            selected: None,
            drawn: None,
            match_tile: None,
            recent_index: None,
            dora_indicators: &[],
            aka_dora: false,
        }
    }
}

pub fn meld_line(
    meld: &Meld,
    theme: &Theme,
    dora_indicators: &[Tile],
    aka_dora: bool,
) -> Line<'static> {
    let kind = meld_kind_label(meld.kind());
    let highlight = TileHighlight {
        dora_indicators,
        aka_dora,
        ..TileHighlight::default()
    };
    let spans: Vec<Span<'static>> = std::iter::once(Span::styled(
        format!("{kind}: "),
        Style::default().fg(theme.primary),
    ))
    .chain(
        meld.tiles()
            .iter()
            .map(|tile| tile_span(*tile, theme, highlight)),
    )
    .collect();
    Line::from(spans)
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
