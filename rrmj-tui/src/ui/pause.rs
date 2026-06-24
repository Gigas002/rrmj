use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::PauseItem;
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_pause_popup(
    frame: &mut ratatui::Frame,
    area: Rect,
    selected: PauseItem,
    theme: &Theme,
) {
    let popup = popup::open_popup(frame, area, 60, 40);
    let lines: Vec<Line> = PauseItem::ALL
        .iter()
        .map(|item| {
            let prefix = if *item == selected { "> " } else { "  " };
            let style = if *item == selected {
                theme.menu_selected_style()
            } else {
                Style::default().fg(theme.primary)
            };
            Line::from(Span::styled(format!("{prefix}{}", item.label()), style))
        })
        .collect();

    let mut body_lines = vec![Line::from("Game paused"), Line::from("")];
    body_lines.extend(lines);
    body_lines.push(Line::from(""));
    body_lines.push(Line::from(Span::styled(
        "↑/↓ navigate  enter select  esc resume",
        Style::default().fg(theme.muted),
    )));

    let widget = Paragraph::new(body_lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Pause"),
    );
    frame.render_widget(widget, popup);
}
