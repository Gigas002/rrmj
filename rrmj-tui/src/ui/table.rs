use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, TableMode, phase_label, seat_label, sorted_tiles};
use crate::ui::widgets::{meld_line, tile_span, tiles_line};

pub fn draw_table(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let Some(view) = app.player_view() else {
        return;
    };
    let human = app.human_seat_active();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(6),
            Constraint::Length(3),
        ])
        .split(area);

    let dora: String = view
        .dora_indicators
        .iter()
        .map(|t| format!("[{t}]"))
        .collect();
    let header = format!(
        "{} {}-{} honba {} | {} | dora {dora} | riichi sticks {}",
        view.round_wind.as_str().to_uppercase(),
        view.kyoku,
        view.dealer + 1,
        view.honba,
        phase_label(view.phase),
        view.table_riichi_sticks
    );
    frame.render_widget(
        Paragraph::new(header).block(Block::default().borders(Borders::ALL).title("Table")),
        root[0],
    );

    let table_area = root[1];
    let positions = table_layout(table_area);
    render_seat(frame, positions[2], &view, human, 2, app);
    render_seat(frame, positions[3], &view, human, 3, app);
    render_seat(frame, positions[0], &view, human, 0, app);
    render_seat(frame, positions[1], &view, human, 1, app);

    let action_lines = action_help(app);
    frame.render_widget(
        Paragraph::new(action_lines)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Actions")),
        root[2],
    );

    let status = if app.is_human_pending() {
        format!("Your turn — {}", app.table_mode().label())
    } else {
        "Waiting for opponents…".into()
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(status, Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::raw(app.status()),
        ])),
        root[3],
    );
}

fn table_layout(area: Rect) -> [Rect; 4] {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    let left = cols[0];
    let center = cols[1];
    let right = cols[2];

    let center_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(center);

    [center_rows[1], right, center_rows[0], left]
}

fn render_seat(
    frame: &mut ratatui::Frame,
    area: Rect,
    view: &librrmj::agent::PlayerView,
    human: usize,
    rel: usize,
    app: &App,
) {
    let seat = (human + rel) % 4;
    let seat_view = &view.seats[seat];
    let is_you = rel == 0;
    let is_actor = view.current_actor == seat
        || (view.phase == librrmj::state::HandPhase::Reaction
            && app
                .match_game()
                .and_then(|g| g.hand().pending_reaction_seat())
                == Some(seat));

    let mut lines = vec![
        Line::from(Span::styled(
            seat_label(seat, human),
            if is_actor {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        )),
        Line::from(format!(
            "Score: {}  tiles: {}{}",
            view.scores[seat],
            seat_view.concealed_count,
            if seat_view.riichi { "  RIICHI" } else { "" }
        )),
    ];

    for meld in &seat_view.melds {
        lines.push(meld_line(meld));
    }

    let discards = sorted_tiles(&seat_view.discards);
    if !discards.is_empty() {
        lines.push(Line::from(Span::raw("River:")));
        lines.push(tiles_line(&discards, None));
    }

    if is_you {
        let mut hand = view.own_concealed.clone();
        hand.sort();
        let selected = pick_selected_index(app, &hand);
        lines.push(Line::from(Span::styled(
            "Hand:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(tiles_line(&hand, selected));
    }

    let title = if seat == view.dealer {
        "Dealer"
    } else {
        "Seat"
    };
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(if is_you {
                Alignment::Center
            } else {
                Alignment::Left
            })
            .block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}

fn pick_selected_index(app: &App, hand: &[librrmj::tile::Tile]) -> Option<usize> {
    let menu = app.action_menu();
    match app.table_mode() {
        TableMode::PickDiscard => menu
            .discards
            .get(app.tile_index())
            .and_then(|t| hand.iter().position(|h| h == t)),
        TableMode::PickRiichi => menu
            .riichi
            .get(app.tile_index())
            .and_then(|t| hand.iter().position(|h| h == t)),
        TableMode::PickClosedKan => menu
            .closed_kans
            .get(app.tile_index())
            .and_then(|t| hand.iter().position(|h| h == t)),
        _ => None,
    }
}

fn action_help(app: &App) -> Vec<Line<'static>> {
    if !app.is_human_pending() {
        return vec![Line::from("Opponents are playing…")];
    }
    let menu = app.action_menu();
    let binds = app.keybinds();
    let mut lines = vec![];

    if menu.is_reaction() {
        if menu.can_ron {
            lines.push(bind_line("Ron", binds, crate::input::BindAction::Ron));
        }
        if menu.can_pon {
            lines.push(bind_line("Pon", binds, crate::input::BindAction::Pon));
        }
        if !menu.chi.is_empty() {
            lines.push(bind_line("Chi", binds, crate::input::BindAction::Chi));
            for (i, chi) in menu.chi.iter().enumerate() {
                let sel = app.table_mode() == TableMode::PickChi && app.chi_index() == i;
                let mut spans = vec![
                    Span::raw(if sel { "  > " } else { "    " }),
                    Span::raw(format!("chi {}: ", i + 1)),
                ];
                spans.extend(chi.iter().map(|t| tile_span(*t, false)));
                lines.push(Line::from(spans));
            }
        }
        if menu.can_open_kan {
            lines.push(bind_line(
                "Open kan",
                binds,
                crate::input::BindAction::OpenKan,
            ));
        }
        if menu.can_pass {
            lines.push(bind_line("Pass", binds, crate::input::BindAction::Pass));
        }
        return lines;
    }

    if menu.can_tsumo {
        lines.push(bind_line("Tsumo", binds, crate::input::BindAction::Tsumo));
    }
    if !menu.riichi.is_empty() {
        lines.push(bind_line("Riichi", binds, crate::input::BindAction::Riichi));
    }
    if !menu.closed_kans.is_empty() {
        lines.push(bind_line(
            "Closed kan",
            binds,
            crate::input::BindAction::ClosedKan,
        ));
    }
    if menu.can_abort_nine_terminals {
        lines.push(bind_line(
            "Abort (9 terminals)",
            binds,
            crate::input::BindAction::AbortNineTerminals,
        ));
    }
    if !menu.discards.is_empty() {
        lines.push(bind_line(
            "Discard",
            binds,
            crate::input::BindAction::Discard,
        ));
        if matches!(
            app.table_mode(),
            TableMode::PickDiscard | TableMode::PickRiichi | TableMode::PickClosedKan
        ) {
            lines.push(Line::from("←/→ select tile, enter confirm, esc cancel"));
        }
    }
    lines
}

fn bind_line(
    label: &str,
    binds: &crate::input::Keybinds,
    action: crate::input::BindAction,
) -> Line<'static> {
    Line::from(format!(
        "{} ({})",
        label,
        crate::input::format_chord(binds.chord(action))
    ))
}
