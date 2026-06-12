use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, ReplayReview, phase_label, seat_label};
use crate::theme::Theme;
use crate::ui::board::{PlayfieldContext, draw_playfield};

pub fn draw_replay_review(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(review) = app.replay_review() else {
        return;
    };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(14),
            Constraint::Length(3),
            Constraint::Min(8),
        ])
        .split(area);

    draw_board(frame, root[0], review, theme);
    draw_status_bar(frame, root[1], review, theme);
    draw_info_panel(frame, root[2], review, theme);
}

fn draw_board(frame: &mut ratatui::Frame, area: Rect, review: &ReplayReview, theme: &Theme) {
    let view = librrmj::agent::PlayerView::from_game(&review.match_game, review.view_seat);
    let mut sorted_hand = view.own_concealed.clone();
    sorted_hand.sort();

    let ctx = PlayfieldContext {
        view: &view,
        human: review.view_seat,
        theme,
        live_remaining: review.match_game.hand().wall().live_remaining(),
        turn_seat: None,
        selected_hand: None,
        drawn_hand: sorted_hand
            .iter()
            .rposition(|t| Some(*t) == view.turn.drawn_tile()),
        highlight_tile: None,
        recent_discard: crate::ui::table::recent_opponent_discard(&view, review.view_seat),
        sorted_hand: &sorted_hand,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_style())
        .title("Final table");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    draw_playfield(frame, inner, &ctx);
}

fn draw_status_bar(frame: &mut ratatui::Frame, area: Rect, review: &ReplayReview, theme: &Theme) {
    let hand = review.match_game.hand();
    let text = format!(
        "View: {} · {} · {} events · esc back",
        seat_label(review.view_seat, review.view_seat),
        phase_label(hand.phase()),
        review.recording.events.len(),
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, theme.status_style()))),
        area,
    );
}

fn draw_info_panel(frame: &mut ratatui::Frame, area: Rect, review: &ReplayReview, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    draw_meta_panel(frame, chunks[0], review, theme);
    draw_event_log(frame, chunks[1], review, theme);
}

fn draw_meta_panel(frame: &mut ratatui::Frame, area: Rect, review: &ReplayReview, theme: &Theme) {
    let rec = &review.recording;
    let mut lines = vec![
        Line::from(Span::styled(review.title(), theme.status_style())),
        Line::from(""),
        Line::from(format!("Seed: {}", rec.seed)),
        Line::from(format!(
            "Hands played: {} · ended {} kyoku",
            rec.hand_index, rec.kyoku
        )),
        Line::from(format!(
            "Final round: {} · honba {}",
            rec.round_wind.as_str(),
            rec.honba
        )),
        Line::from(""),
        Line::from("Final scores:"),
    ];
    for (seat, score) in rec.scores.iter().enumerate() {
        lines.push(Line::from(format!(
            "  {}: {score}",
            crate::app::NewGameSetup::seat_name(seat)
        )));
    }
    if let Some(desc) = rec.meta.description.as_ref()
        && !desc.is_empty()
    {
        lines.push(Line::from(""));
        lines.push(Line::from(desc.clone()));
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.block_style())
                .title("Match summary"),
        ),
        area,
    );
}

fn draw_event_log(frame: &mut ratatui::Frame, area: Rect, review: &ReplayReview, theme: &Theme) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let events = review.event_lines();
    let visible: Vec<Line> = events
        .iter()
        .skip(review.event_scroll)
        .take(inner_height)
        .map(|line| {
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.primary),
            ))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(visible).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.block_style())
                .title(format!("Event log (↑/↓ scroll, {} total)", events.len())),
        ),
        area,
    );
}
