use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, MainMenuMode, Screen};
use crate::input::{BindAction, action_label, format_chord};
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_help_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let popup = popup::open_popup(frame, area, 85, 80);

    let mut lines = vec![
        Line::from("Keybind help — press h or esc to close"),
        Line::from(""),
    ];
    for (action, chord) in app.keybinds().entries() {
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(action),
            format_chord(chord)
        )));
    }

    if app.screen() == Screen::Table {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "At the table",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(BindAction::Back),
            format_chord(app.keybinds().chord(BindAction::Back)),
        )));
        lines.push(Line::from(
            "  Pause menu: resume, save game, return to main menu, quit",
        ));
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(BindAction::MainMenu),
            format_chord(app.keybinds().chord(BindAction::MainMenu)),
        )));
        lines.push(Line::from(
            "  Return to main menu without saving (use pause menu to save)",
        ));
    }

    if app.resume_setup_open() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Resume setup (saved game or scenario)",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(
            "  Choose your seat; CPUs keep AI settings from the save file.",
        ));
        lines.push(Line::from(
            "  On match end the save is promoted to a finished replay in place.",
        ));
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(BindAction::Back),
            format_chord(app.keybinds().chord(BindAction::Back)),
        )));
    }

    if app.main_menu_mode() == MainMenuMode::Scenarios {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Scenarios (community packs)",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(
            "  f — cycle tag filter  i — import JSON from path",
        ));
    }

    #[cfg(feature = "debug-menu")]
    if app.main_menu_mode() == MainMenuMode::Debug {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Debug scenarios (repo CI fixtures)",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(
            "  examples/scenarios/*.json — f filter  i import",
        ));
    }

    if app.screen() == Screen::ReplayReview {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Replay (observe only)",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(BindAction::Back),
            format_chord(app.keybinds().chord(BindAction::Back)),
        )));
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(BindAction::TilePrev),
            format_chord(app.keybinds().chord(BindAction::TilePrev)),
        )));
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(BindAction::TileNext),
            format_chord(app.keybinds().chord(BindAction::TileNext)),
        )));
        lines.push(Line::from(format!(
            "{:28} {}",
            action_label(BindAction::MenuCycle),
            format_chord(app.keybinds().chord(BindAction::MenuCycle)),
        )));
        lines.push(Line::from("  space — play / pause auto-step"));
        lines.push(Line::from("  home / end — jump to start / end"));
        lines.push(Line::from("  n / b — next / previous hand"));
        lines.push(Line::from("  1-4 — view seat (full hand)"));
    }

    let widget = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Help"),
    );
    frame.render_widget(widget, popup);
}
