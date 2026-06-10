use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_hand_result_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(result) = app.hand_result() else {
        return;
    };
    let popup = popup::open_popup(frame, area, 65, 55);

    let mut lines = vec![Line::from(result.title.clone())];
    for line in &result.lines {
        lines.push(Line::from(line.clone()));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Press Enter to continue"));

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.block_style())
                .title("Hand result"),
        ),
        popup,
    );
}

pub fn draw_match_summary_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(scores) = app.match_summary() else {
        return;
    };
    let popup = popup::open_popup(frame, area, 55, 45);

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
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.block_style())
                .title("Match over"),
        ),
        popup,
    );
}
