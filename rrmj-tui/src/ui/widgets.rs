use librrmj::hand::Meld;
use librrmj::tile::Tile;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn tile_span(tile: Tile, selected: bool) -> Span<'static> {
    let label = format!("[{tile}]");
    let style = if tile.is_red() {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };
    let style = if selected {
        style.add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else {
        style
    };
    Span::styled(label, style)
}

pub fn tiles_line(tiles: &[Tile], selected: Option<usize>) -> Line<'static> {
    let spans: Vec<Span> = tiles
        .iter()
        .enumerate()
        .map(|(i, t)| tile_span(*t, selected == Some(i)))
        .collect();
    Line::from(spans)
}

pub fn meld_line(meld: &Meld) -> Line<'static> {
    let kind = match meld.kind() {
        librrmj::hand::MeldKind::Chi => "chi",
        librrmj::hand::MeldKind::Pon => "pon",
        librrmj::hand::MeldKind::OpenKan => "minkan",
        librrmj::hand::MeldKind::ClosedKan => "ankan",
        librrmj::hand::MeldKind::AddedKan => "kakan",
    };
    let tiles: String = meld.tiles().iter().map(|t| format!("[{t}]")).collect();
    Line::from(format!("{kind}: {tiles}"))
}
