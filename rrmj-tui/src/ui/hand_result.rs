use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn draw_hand_result(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let Some(result) = app.hand_result() else {
        return;
    };
    let mut lines = vec![Line::from(result.title.clone())];
    for line in &result.lines {
        lines.push(Line::from(line.clone()));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Press Enter to continue"));

    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Hand result")),
        area,
    );
}

pub fn draw_match_summary(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let Some(scores) = app.match_summary() else {
        return;
    };
    let lines: Vec<Line> = scores
        .iter()
        .enumerate()
        .map(|(seat, score)| {
            Line::from(format!(
                "{}: {score}",
                crate::app::NewGameSetup::seat_name(seat)
            ))
        })
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from("Press Enter to return to menu")))
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Match over")),
        area,
    );
}
