use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::app::{App, MainMenuMode};
use crate::theme::Theme;

pub fn draw_scenario_lines(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let entries = app.filtered_scenario_entries();
    let filter = app
        .scenario_filter_tag()
        .map(|t| format!("[{t}] "))
        .unwrap_or_default();
    if entries.is_empty() {
        let dir = app.config().resolved_scenarios_dir();
        return vec![
            Line::from("No scenarios found."),
            Line::from(Span::styled(
                format!("Directory: {}", dir.display()),
                Style::default().fg(theme.muted),
            )),
            Line::from(Span::styled(
                "Set scenarios_dir in config.toml or copy JSON packs there.",
                Style::default().fg(theme.muted),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "i import from path  Enter or Esc to return",
                Style::default().fg(theme.muted),
            )),
        ];
    }

    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let prefix = if i == app.menu_index() { "> " } else { "  " };
            let style = if i == app.menu_index() {
                theme.menu_selected_style()
            } else {
                Style::default().fg(theme.primary)
            };
            let tags = if entry.tags.is_empty() {
                String::new()
            } else {
                format!("[{}] ", entry.tags.join(","))
            };
            Line::from(vec![
                Span::styled(format!("{prefix}{tags}{}", entry.title), style),
                Span::raw(" — "),
                Span::styled(entry.description.clone(), Style::default().fg(theme.muted)),
            ])
        })
        .chain(
            std::iter::once(Line::from("")).chain(std::iter::once(Line::from(Span::styled(
                format!("{filter}f cycle tag filter  i import from path"),
                Style::default().fg(theme.muted),
            )))),
        )
        .collect()
}

pub const fn is_scenarios_menu_mode(mode: MainMenuMode) -> bool {
    matches!(mode, MainMenuMode::Scenarios)
}
