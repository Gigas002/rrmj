use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_recommendations_popup(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    theme: &Theme,
) {
    let popup = popup::open_popup(frame, area, 80, 70);

    let mut lines = vec![
        Line::from(Span::styled(
            "Win path recommendations",
            theme.title_style(),
        )),
        Line::from(Span::styled(
            "Sorted by expected points, then shanten.",
            Style::default().fg(theme.muted),
        )),
        Line::from(""),
    ];

    let candidates = app.recommendations();
    if candidates.is_empty() {
        lines.push(Line::from("No candidate paths for the current hand."));
    } else {
        for (index, path) in candidates.iter().enumerate() {
            lines.push(Line::from(Span::styled(
                format!(
                    "{}. {} — {}",
                    index + 1,
                    path.shanten_label(),
                    path.win_type_label()
                ),
                theme.status_style(),
            )));
            lines.push(Line::from(path.summary_line()));
            if let Some(tile) = path.win_tile {
                lines.push(Line::from(format!("   Wait: {tile}")));
            }
            let mut dora = Vec::new();
            if path.dora > 0 {
                dora.push(format!("dora {}", path.dora));
            }
            if path.ura_dora > 0 {
                dora.push(format!("ura {}", path.ura_dora));
            }
            if path.aka_dora > 0 {
                dora.push(format!("aka {}", path.aka_dora));
            }
            if !dora.is_empty() {
                lines.push(Line::from(format!("   {}", dora.join(", "))));
            }
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(Span::styled(
        "e / esc to close  ↑↓ scroll",
        Style::default().fg(theme.muted),
    )));

    let scroll = app.recommendations_scroll();
    let visible: Vec<Line> = lines
        .into_iter()
        .skip(scroll)
        .take(popup.height.saturating_sub(2) as usize)
        .collect();

    frame.render_widget(
        Paragraph::new(visible).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.block_style())
                .title("Recommendations"),
        ),
        popup,
    );
}

pub fn recommendation_line_count(app: &App) -> usize {
    let mut count = 4;
    let candidates = app.recommendations();
    if candidates.is_empty() {
        count += 1;
    } else {
        for path in candidates {
            count += 2;
            if path.win_tile.is_some() {
                count += 1;
            }
            if path.dora > 0 || path.ura_dora > 0 || path.aka_dora > 0 {
                count += 1;
            }
            count += 1;
        }
    }
    count += 1;
    count
}
