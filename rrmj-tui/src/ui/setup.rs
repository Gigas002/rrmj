use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, NewGameSetup, SetupField};
use librrmj::agent::PlayerSlot;

pub fn draw_setup(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let Some(setup) = app.setup() else {
        return;
    };

    let mut lines = Vec::new();
    lines.push(Line::from("Configure seats, then confirm."));
    lines.push(Line::from(""));

    for seat in 0..4 {
        let slot = setup.slots[seat];
        let slot_label = match slot {
            PlayerSlot::Human => "Human",
            PlayerSlot::Cpu => "CPU",
            PlayerSlot::Remote => "Remote",
        };
        let diff = crate::app::difficulty_label(setup.difficulties[seat]);
        let selected_type = matches!(setup.selected, SetupField::SeatType(s) if s == seat);
        let selected_diff = matches!(setup.selected, SetupField::SeatDifficulty(s) if s == seat);
        let type_style = if selected_type {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let diff_style = if selected_diff {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::raw(format!("{}: ", NewGameSetup::seat_name(seat))),
            Span::styled(format!("[{slot_label}]"), type_style),
            Span::raw("  "),
            Span::styled(format!("({diff})"), diff_style),
        ]));
    }

    lines.push(Line::from(""));
    let human_style = if setup.selected == SetupField::HumanSeat {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![
        Span::raw("Your seat: "),
        Span::styled(
            NewGameSetup::seat_name(setup.human_seat).to_string(),
            human_style,
        ),
    ]));

    let confirm_style = if setup.selected == SetupField::Confirm {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(Span::styled("> Start match", confirm_style)));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    let body = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("New game setup"),
    );
    frame.render_widget(body, chunks[0]);

    let footer =
        Paragraph::new("↑/↓ navigate  space toggle  tab cycle  enter confirm/start  esc back");
    frame.render_widget(footer, chunks[1]);
}
