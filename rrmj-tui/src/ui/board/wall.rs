use librrmj::tile::Tile;
use librrmj::wall::LIVE_WALL_AFTER_DEAL;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme::Theme;
use crate::ui::render::tile_label;

pub fn wall_lines(
    live_remaining: usize,
    dora: &[Tile],
    riichi_sticks: u8,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let bar_len = 20usize;
    let filled = live_remaining
        .saturating_mul(bar_len)
        .div_ceil(LIVE_WALL_AFTER_DEAL)
        .min(bar_len);
    let bar: String = (0..bar_len)
        .map(|i| if i < filled { '█' } else { '░' })
        .collect();
    lines.push(Line::from(vec![
        Span::styled("Wall ", theme.title_style()),
        Span::styled(
            format!("{bar} {live_remaining}"),
            Style::default().fg(theme.primary),
        ),
    ]));

    if !dora.is_empty() {
        let labels: Vec<String> = dora.iter().map(|t| tile_label(*t)).collect();
        lines.push(Line::from(vec![
            Span::styled("Dora ", theme.dora_style()),
            Span::styled(labels.join(" "), Style::default().fg(theme.primary)),
        ]));
    }

    if riichi_sticks > 0 {
        lines.push(Line::from(Span::styled(
            format!("Riichi sticks ×{riichi_sticks} (1000 each)"),
            theme.riichi_style(false),
        )));
    }

    lines.push(Line::from(Span::styled(
        "Dead wall ░░░░░░░░░░░░░░",
        Style::default().fg(theme.muted).add_modifier(Modifier::DIM),
    )));

    lines
}
