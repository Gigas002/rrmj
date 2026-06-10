use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, seat_label};
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_scores_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(view) = app.player_view() else {
        return;
    };
    let human = app.human_seat_active();
    let popup = popup::open_popup(frame, area, 55, 45);

    let header = format!(
        "{} round {}-{}  honba {}  riichi sticks {}",
        view.round_wind.as_str().to_uppercase(),
        view.kyoku,
        view.dealer + 1,
        view.honba,
        view.table_riichi_sticks,
    );

    let mut lines = vec![
        Line::from(Span::styled(header, theme.title_style())),
        Line::from(""),
    ];

    let mut ranked: Vec<(usize, i32)> = view.scores.iter().copied().enumerate().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    for (rank, (seat, score)) in ranked.iter().enumerate() {
        let place = rank + 1;
        let name = seat_label(*seat, human);
        let mut tags = Vec::new();
        if *seat == view.dealer {
            tags.push("dealer");
        }
        if *seat == human {
            tags.push("you");
        }
        let tag_suffix = if tags.is_empty() {
            String::new()
        } else {
            format!("  ({})", tags.join(", "))
        };
        let style = if *seat == human {
            theme.menu_selected_style()
        } else {
            Style::default().fg(theme.primary)
        };
        lines.push(Line::from(vec![
            Span::raw(format!("{place}. ")),
            Span::styled(format!("{name}{tag_suffix}"), style),
            Span::raw(format!("  {:>6}", score)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "s or esc to close",
        Style::default().fg(theme.muted),
    )));

    let widget = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Scores"),
    );
    frame.render_widget(widget, popup);
}
