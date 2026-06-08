use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::App;
use crate::input::{action_label, format_chord};

pub fn draw_help(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    frame.render_widget(Clear, area);

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

    let widget = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(widget, area);
}
