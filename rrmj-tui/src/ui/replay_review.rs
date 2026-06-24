use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
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
    let game = review.player.game();
    let view = librrmj::agent::PlayerView::from_game(game, review.view_seat);
    let mut sorted_hand = view.own_concealed.clone();
    sorted_hand.sort();

    let recent_discard = review
        .recent_discard_highlight()
        .or_else(|| crate::ui::table::recent_opponent_discard(&view, review.view_seat));

    let ctx = PlayfieldContext {
        view: &view,
        human: review.view_seat,
        theme,
        live_remaining: game.hand().wall().live_remaining(),
        turn_seat: Some(game.hand().current_actor()),
        selected_hand: None,
        drawn_hand: sorted_hand
            .iter()
            .rposition(|t| Some(*t) == view.turn.drawn_tile()),
        highlight_tile: None,
        recent_discard,
        sorted_hand: &sorted_hand,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_style())
        .title("Replay");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    draw_playfield(frame, inner, &ctx);
}

fn draw_status_bar(frame: &mut ratatui::Frame, area: Rect, review: &ReplayReview, theme: &Theme) {
    let hand = review.player.game().hand();
    let text = format!(
        "{} · View {} · {} · {} · space play/pause · ←/→ step · tab seat · esc back",
        review.status_text(),
        seat_label(review.view_seat, review.view_seat),
        phase_label(hand.phase()),
        review.title(),
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
    let rec = review.player.recording();
    let mut lines = vec![
        Line::from(Span::styled(review.title(), theme.status_style())),
        Line::from(""),
        Line::from(format!("Seed: {}", rec.seed)),
        Line::from(format!("Hands in log: {}", rec.hand_index)),
        Line::from(format!(
            "Round at save: {} · honba {}",
            rec.round_wind.as_str(),
            rec.honba
        )),
        Line::from(""),
        Line::from("Scores at cursor:"),
    ];
    for (seat, score) in review.player.game().scores().iter().enumerate() {
        lines.push(Line::from(format!(
            "  {}: {score}",
            crate::app::NewGameSetup::seat_name(seat)
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Playback:"));
    lines.push(Line::from("  home/end — start/end"));
    lines.push(Line::from("  n / b — next/prev hand"));
    lines.push(Line::from("  1-4 — view seat"));
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
                .title("Replay info"),
        ),
        area,
    );
}

fn draw_event_log(frame: &mut ratatui::Frame, area: Rect, review: &ReplayReview, theme: &Theme) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let events = review.event_lines();
    let cursor = review.cursor_line_index();
    let visible: Vec<Line> = events
        .iter()
        .enumerate()
        .skip(review.event_scroll)
        .take(inner_height)
        .map(|(index, line)| {
            let style = if index == cursor {
                theme.menu_selected_style().add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.primary)
            };
            Line::from(Span::styled(line.clone(), style))
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
