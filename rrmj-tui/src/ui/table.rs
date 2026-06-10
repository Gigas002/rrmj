use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, TableMode};
use crate::theme::Theme;
use crate::ui::board::{PlayfieldContext, draw_playfield};
use crate::ui::widgets::tile_span;

pub fn draw_table(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(view) = app.player_view() else {
        return;
    };
    let human = app.human_seat_active();
    let mut sorted_hand = view.own_concealed.clone();
    sorted_hand.sort();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(14),
            Constraint::Length(6),
            Constraint::Length(3),
        ])
        .split(area);

    let selected_hand = pick_selected_index(app, &sorted_hand);
    let drawn_hand = drawn_hand_index(&sorted_hand, view.last_draw);
    let ctx = PlayfieldContext {
        view: &view,
        human,
        theme,
        live_remaining: app.wall_remaining().unwrap_or(0),
        turn_seat: app.turn_highlight_seat(),
        selected_hand,
        drawn_hand,
        highlight_tile: highlighted_hand_tile(&sorted_hand, selected_hand, drawn_hand),
        sorted_hand: &sorted_hand,
    };
    draw_playfield(frame, root[0], &ctx);

    frame.render_widget(
        Paragraph::new(action_help(app, theme))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.block_style())
                    .title("Actions"),
            ),
        root[1],
    );

    let status = if app.is_human_pending() {
        format!("Your decision — {}", app.table_mode().label())
    } else {
        "Waiting…  (esc pause)".into()
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(status, theme.status_style()),
            Span::raw("  "),
            Span::raw(app.status()),
        ])),
        root[2],
    );
}

fn pick_selected_index(app: &App, hand: &[librrmj::tile::Tile]) -> Option<usize> {
    let menu = app.action_menu();
    let (picker_tiles, picker_index) = match app.table_mode() {
        TableMode::PickDiscard => (&menu.discards, app.tile_index()),
        TableMode::PickRiichi => (&menu.riichi, app.tile_index()),
        TableMode::PickClosedKan => (&menu.closed_kans, app.tile_index()),
        _ => return None,
    };
    picker_index_to_hand_index(hand, picker_tiles, picker_index)
}

fn drawn_hand_index(
    hand: &[librrmj::tile::Tile],
    drawn: Option<librrmj::tile::Tile>,
) -> Option<usize> {
    let drawn = drawn?;
    hand.iter().rposition(|t| *t == drawn)
}

fn highlighted_hand_tile(
    hand: &[librrmj::tile::Tile],
    selected: Option<usize>,
    drawn: Option<usize>,
) -> Option<librrmj::tile::Tile> {
    selected.or(drawn).and_then(|i| hand.get(i).copied())
}

/// Maps a sorted legal-action picker entry to the matching hand index (duplicates included).
fn picker_index_to_hand_index(
    hand: &[librrmj::tile::Tile],
    picker_tiles: &[librrmj::tile::Tile],
    picker_index: usize,
) -> Option<usize> {
    let tile = picker_tiles.get(picker_index)?;
    let occurrence = picker_tiles
        .iter()
        .take(picker_index + 1)
        .filter(|&&t| t == *tile)
        .count();
    hand.iter()
        .enumerate()
        .filter(|(_, h)| **h == *tile)
        .nth(occurrence - 1)
        .map(|(i, _)| i)
}

fn action_help(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    if !app.is_human_pending() {
        return vec![Line::from(Span::styled(
            "Waiting…  (esc pause)",
            theme.muted_style(),
        ))];
    }
    let binds = app.keybinds();
    let menu = app.action_menu();
    let mut lines = vec![];
    if let Some(label) = app.human_decision_timer_label() {
        lines.push(Line::from(Span::styled(
            format!("Time left: {label}"),
            theme.status_style(),
        )));
        lines.push(Line::from(""));
    }

    if menu.is_reaction() {
        if menu.can_ron {
            lines.push(action_line(
                "Ron",
                binds,
                crate::input::BindAction::Ron,
                Style::default().fg(theme.danger),
            ));
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
                spans.extend(
                    chi.iter()
                        .map(|t| tile_span(*t, theme, false, false, false)),
                );
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
            lines.push(action_line(
                "Pass",
                binds,
                crate::input::BindAction::Pass,
                Style::default().fg(theme.safe),
            ));
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
            lines.push(Line::from(Span::styled(
                "←/→ select tile, enter confirm, esc cancel",
                theme.muted_style(),
            )));
        }
    }
    lines
}

fn bind_line(
    label: &str,
    binds: &crate::input::Keybinds,
    action: crate::input::BindAction,
) -> Line<'static> {
    action_line(label, binds, action, Style::default())
}

fn action_line(
    label: &str,
    binds: &crate::input::Keybinds,
    action: crate::input::BindAction,
    style: Style,
) -> Line<'static> {
    Line::from(Span::styled(
        format!(
            "{} ({})",
            label,
            crate::input::format_chord(binds.chord(action))
        ),
        style,
    ))
}

#[cfg(test)]
mod tests {
    use librrmj::tile::Tile;

    use super::{drawn_hand_index, picker_index_to_hand_index};

    #[test]
    fn picker_index_steps_through_duplicate_tiles() {
        let hand = [
            Tile::man(8),
            Tile::man(8),
            Tile::sou(1),
            Tile::sou(2),
            Tile::sou(3),
        ];
        let discards = hand.to_vec();
        assert_eq!(picker_index_to_hand_index(&hand, &discards, 0), Some(0));
        assert_eq!(picker_index_to_hand_index(&hand, &discards, 1), Some(1));
        assert_eq!(picker_index_to_hand_index(&hand, &discards, 2), Some(2));
    }

    #[test]
    fn drawn_tile_highlights_rightmost_copy() {
        let hand = [Tile::man(8), Tile::man(8), Tile::sou(1)];
        assert_eq!(drawn_hand_index(&hand, Some(Tile::man(8))), Some(1));
    }
}
