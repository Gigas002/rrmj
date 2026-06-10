use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, LoadGameSetup, LoadSetupField};
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_load_setup_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(load) = app.load_setup() else {
        return;
    };
    let popup = popup::open_popup(frame, area, 80, 76);
    draw_load_setup_content(frame, popup, load, theme);
}

fn draw_load_setup_content(
    frame: &mut ratatui::Frame,
    area: Rect,
    load: &LoadGameSetup,
    theme: &Theme,
) {
    let saved = LoadGameSetup::seat_name(load.saved_human_seat);
    let mut lines = vec![
        Line::from(format!("Save: {}", load.entry.label)),
        Line::from(""),
        Line::from(vec![
            Span::raw("Originally played as: "),
            Span::styled(saved, theme.status_style().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(
            "Resume in that seat unless you want to study as another player.",
            Style::default().fg(theme.muted),
        )),
        Line::from(""),
    ];

    let seat_style = if load.selected == LoadSetupField::HumanSeat {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    let seat_label = LoadGameSetup::seat_name(load.selected_seat);
    let hint = if load.using_saved_seat() {
        " (recommended)"
    } else {
        " (study / alternate seat)"
    };
    lines.push(Line::from(vec![
        Span::raw("Your seat: "),
        Span::styled(format!("{seat_label}{hint}"), seat_style),
    ]));

    let cpu_style = if load.selected == LoadSetupField::CpuStepDelay {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(vec![
        Span::raw("CPU decision delay: "),
        Span::styled(crate::timers::label_cpu(load.cpu_step_delay_ms), cpu_style),
    ]));

    let turn_style = if load.selected == LoadSetupField::TurnTimer {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(vec![
        Span::raw("Turn timer: "),
        Span::styled(crate::timers::label_turn(load.turn_timer_ms), turn_style),
    ]));

    let response_style = if load.selected == LoadSetupField::ResponseTimer {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(vec![
        Span::raw("Call response timer: "),
        Span::styled(
            crate::timers::label_response(load.response_timer_ms),
            response_style,
        ),
    ]));

    let confirm_style = if load.selected == LoadSetupField::Confirm {
        theme.status_style().add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(Span::styled("> Load game", confirm_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓ navigate  space/tab cycle  enter load  esc back",
        Style::default().fg(theme.muted),
    )));

    let body = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Load game — choose seat"),
    );
    frame.render_widget(body, area);
}
