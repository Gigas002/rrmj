use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::input::{action_label, format_chord};
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_help_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let popup = popup::open_popup(frame, area, 85, 80);

    let mut lines = vec![
        Line::from("Keybind help — press h or esc to close"),
        Line::from(""),
    ];
    for (action, chord) in app.keybinds().entries() {
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(action),
            format_chord(chord)
        )));
    }

    let widget = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Help"),
    );
    frame.render_widget(widget, popup);
}
