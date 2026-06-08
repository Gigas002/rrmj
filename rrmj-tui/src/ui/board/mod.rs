mod wall;

use librrmj::agent::PlayerView;
use librrmj::tile::Tile;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{phase_label, seat_label};
use crate::theme::Theme;
use crate::ui::board::wall::wall_lines;
use crate::ui::widgets::{meld_line, riichi_badge, tiles_line};

pub struct PlayfieldContext<'a> {
    pub view: &'a PlayerView,
    pub human: usize,
    pub theme: &'a Theme,
    pub live_remaining: usize,
    pub actor_seat: Option<usize>,
    pub selected_hand: Option<usize>,
    pub sorted_hand: &'a [Tile],
}

pub fn draw_playfield(frame: &mut ratatui::Frame, area: Rect, ctx: &PlayfieldContext<'_>) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(37),
            Constraint::Percentage(35),
        ])
        .split(area);

    draw_seat_panel(frame, rows[0], ctx, 2, Alignment::Center);
    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(rows[1]);
    draw_seat_panel(frame, mid[0], ctx, 3, Alignment::Left);
    draw_center_panel(frame, mid[1], ctx);
    draw_seat_panel(frame, mid[2], ctx, 1, Alignment::Right);
    draw_seat_panel(frame, rows[2], ctx, 0, Alignment::Center);
}

fn draw_center_panel(frame: &mut ratatui::Frame, area: Rect, ctx: &PlayfieldContext<'_>) {
    let header = format!(
        "{} {}-{} honba {}",
        ctx.view.round_wind.as_str().to_uppercase(),
        ctx.view.kyoku,
        ctx.view.dealer + 1,
        ctx.view.honba,
    );
    let mut lines = vec![
        Line::from(Span::styled(header, ctx.theme.title_style())),
        Line::from(Span::styled(
            phase_label(ctx.view.phase),
            ctx.theme.status_style(),
        )),
        Line::from(""),
    ];
    lines.extend(wall_lines(
        ctx.live_remaining,
        &ctx.view.dora_indicators,
        ctx.view.table_riichi_sticks,
        ctx.theme,
    ));
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(ctx.theme.block_style())
                    .title("Table"),
            ),
        area,
    );
}

fn draw_seat_panel(
    frame: &mut ratatui::Frame,
    area: Rect,
    ctx: &PlayfieldContext<'_>,
    rel: usize,
    align: Alignment,
) {
    let seat = (ctx.human + rel) % 4;
    let seat_view = &ctx.view.seats[seat];
    let is_you = rel == 0;
    let is_actor = ctx.actor_seat == Some(seat);

    let mut lines = seat_header_lines(ctx, seat, is_actor);
    lines.push(Line::from(""));

    for meld in &seat_view.melds {
        lines.push(meld_line(meld, ctx.theme));
    }
    if !seat_view.melds.is_empty() {
        lines.push(Line::from(""));
    }

    if !seat_view.discards.is_empty() {
        lines.push(Line::from(Span::styled("River", ctx.theme.muted_style())));
        lines.push(tiles_line(&seat_view.discards, ctx.theme, None));
        lines.push(Line::from(""));
    }

    if is_you {
        lines.push(Line::from(Span::styled("Hand", ctx.theme.title_style())));
        lines.push(tiles_line(
            ctx.sorted_hand,
            ctx.theme,
            ctx.selected_hand,
        ));
    } else if seat_view.concealed_count > 0 {
        lines.push(Line::from(Span::styled(
            format!("Hand ({} concealed)", seat_view.concealed_count),
            ctx.theme.muted_style(),
        )));
    }

    let border = if is_actor {
        ctx.theme.actor_style(false)
    } else {
        ctx.theme.block_style()
    };
    let title = if seat == ctx.view.dealer {
        format!("{} · dealer", seat_label(seat, ctx.human))
    } else {
        seat_label(seat, ctx.human)
    };

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(align)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border)
                    .title(title),
            ),
        area,
    );
}

fn seat_header_lines(ctx: &PlayfieldContext<'_>, seat: usize, is_actor: bool) -> Vec<Line<'static>> {
    let seat_view = &ctx.view.seats[seat];
    let name_style = if is_actor {
        ctx.theme.actor_style(false)
    } else {
        Style::default().fg(ctx.theme.primary)
    };
    let mut spans = vec![
        Span::styled(seat_label(seat, ctx.human), name_style),
        Span::raw(format!("  {:>6}", ctx.view.scores[seat])),
    ];
    if seat_view.riichi {
        spans.push(riichi_badge(ctx.theme, false));
    }
    vec![Line::from(spans)]
}
