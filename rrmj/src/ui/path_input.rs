use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_path_input_popup(
    frame: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    prompt: &str,
    path: &str,
    footer: &str,
    theme: &Theme,
) {
    let popup = popup::open_popup(frame, area, 72, 30);
    let lines = vec![
        Line::from(prompt),
        Line::from(""),
        Line::from(Span::styled(path, Style::default().fg(theme.primary))),
        Line::from(""),
        Line::from(Span::styled(footer, Style::default().fg(theme.muted))),
    ];

    let widget = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title(title),
    );
    frame.render_widget(widget, popup);
}
