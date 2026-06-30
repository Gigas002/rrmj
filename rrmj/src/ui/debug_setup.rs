#![cfg(feature = "debug-menu")]

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, DebugScenarioSetup, DebugSetupField};
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_debug_setup_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(setup) = app.debug_setup() else {
        return;
    };
    let popup = popup::open_popup(frame, area, 80, 76);
    draw_debug_setup_content(frame, popup, setup, theme);
}

fn draw_debug_setup_content(
    frame: &mut ratatui::Frame,
    area: Rect,
    setup: &DebugScenarioSetup,
    theme: &Theme,
) {
    let saved = DebugScenarioSetup::seat_name(setup.saved_human_seat);
    let mut lines = vec![
        Line::from(format!("Scenario: {}", setup.entry.title)),
        Line::from(Span::styled(
            setup.entry.description.clone(),
            Style::default().fg(theme.muted),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("Fixture seat: "),
            Span::styled(saved, theme.status_style().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    let seat_style = if setup.selected == DebugSetupField::HumanSeat {
        theme.menu_selected_style()
    } else {
        Style::default().fg(theme.primary)
    };
    let seat_label = DebugScenarioSetup::seat_name(setup.selected_seat);
    let hint = if setup.using_saved_seat() {
        " (recommended)"
    } else {
        " (study / alternate seat)"
    };
    lines.push(Line::from(vec![
        Span::raw("Your seat: "),
        Span::styled(format!("{seat_label}{hint}"), seat_style),
    ]));

    let confirm_style = if setup.selected == DebugSetupField::Confirm {
        theme.status_style().add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.primary)
    };
    lines.push(Line::from(Span::styled("> Launch scenario", confirm_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓ navigate  space/tab cycle seat  enter launch  esc back",
        Style::default().fg(theme.muted),
    )));

    let body = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Debug scenario — choose seat"),
    );
    frame.render_widget(body, area);
}
