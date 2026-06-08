use librrmj::hand::Meld;
use librrmj::tile::Tile;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme::Theme;
use crate::ui::render::{meld_kind_label, tile_label};

pub fn tile_span(tile: Tile, theme: &Theme, selected: bool) -> Span<'static> {
    Span::styled(tile_label(tile), theme.tile_style(tile.is_red(), selected))
}

pub fn tiles_line(tiles: &[Tile], theme: &Theme, selected: Option<usize>) -> Line<'static> {
    Line::from(
        tiles
            .iter()
            .enumerate()
            .map(|(i, t)| tile_span(*t, theme, selected == Some(i)))
            .collect::<Vec<_>>(),
    )
}

pub fn meld_line(meld: &Meld, theme: &Theme) -> Line<'static> {
    let kind = meld_kind_label(meld.kind());
    let tiles: String = meld
        .tiles()
        .iter()
        .map(|t| tile_label(*t))
        .collect();
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
