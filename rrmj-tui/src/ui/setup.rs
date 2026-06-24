use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, NewGameSetup, SetupField};
use crate::theme::Theme;
use crate::ui::popup;
use librrmj::agent::PlayerSlot;

pub fn draw_setup_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(setup) = app.setup() else {
        return;
    };
    let popup = popup::open_popup(frame, area, 80, 78);
    draw_setup_content(frame, popup, setup, theme);
}

fn draw_setup_content(frame: &mut ratatui::Frame, area: Rect, setup: &NewGameSetup, theme: &Theme) {
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
            theme.menu_selected_style()
        } else {
            Style::default().fg(theme.primary)
        };
        let diff_style = if selected_diff {
            theme.menu_selected_style()
        } else {
            Style::default().fg(theme.primary)
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
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(vec![
        Span::raw("Your seat: "),
        Span::styled(
            NewGameSetup::seat_name(setup.human_seat).to_string(),
            human_style,
        ),
    ]));

    let cpu_style = if setup.selected == SetupField::CpuStepDelay {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(vec![
        Span::raw("CPU decision delay: "),
        Span::styled(crate::timers::label_cpu(setup.cpu_step_delay_ms), cpu_style),
    ]));

    let turn_style = if setup.selected == SetupField::TurnTimer {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(vec![
        Span::raw("Turn timer: "),
        Span::styled(crate::timers::label_turn(setup.turn_timer_ms), turn_style),
    ]));

    let response_style = if setup.selected == SetupField::ResponseTimer {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(vec![
        Span::raw("Call response timer: "),
        Span::styled(
            crate::timers::label_response(setup.response_timer_ms),
            response_style,
        ),
    ]));

    let confirm_style = if setup.selected == SetupField::Confirm {
        theme.status_style().add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(Span::styled("> Start match", confirm_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓ navigate  space toggle  tab cycle  enter start  esc back",
        Style::default().fg(theme.muted),
    )));

    let body = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("New game setup"),
    );
    frame.render_widget(body, area);
}
