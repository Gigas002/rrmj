use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, SettingsField, difficulty_label};
use crate::config::theme_names;
use crate::theme::Theme;
use crate::ui::popup;
use crate::ui::widgets::muted_span;

#[cfg(feature = "debug-menu")]
fn debug_menu_active(app: &App) -> bool {
    crate::ui::debug_menu::is_debug_menu_mode(app.main_menu_mode())
}

#[cfg(not(feature = "debug-menu"))]
const fn debug_menu_active(_app: &App) -> bool {
    false
}

fn scenarios_menu_active(app: &App) -> bool {
    crate::ui::scenario_menu::is_scenarios_menu_mode(app.main_menu_mode())
}

fn recording_list_lines(
    app: &App,
    entries: &[crate::save::RecordingEntry],
    empty_message: &str,
    theme: &Theme,
) -> Vec<Line<'static>> {
    if entries.is_empty() {
        return vec![
            Line::from(empty_message.to_string()),
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter or Esc to return",
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
            Line::from(vec![
                Span::styled(format!("{prefix}{}", entry.label), style),
                Span::raw(" — "),
                Span::styled(entry.detail.clone(), Style::default().fg(theme.muted)),
            ])
        })
        .collect()
}

const LOGO: &str = r"
 ██████  ██████  ███    ███   ███
 ██   ██ ██   ██ ████  ████    ██
 ██████  ██████  ██ ████ ██    ██
 ██   ██ ██   ██ ██  ██  ██ ██ ██
 ██   ██ ██   ██ ██      ██  ████
";

pub fn draw_main_menu(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(area);

    let logo = Paragraph::new(LOGO)
        .style(Style::default().fg(theme.logo))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.block_style())
                .title("rrmj"),
        );
    frame.render_widget(logo, chunks[0]);

    use crate::app::MainMenuMode;

    let lines: Vec<Line> = if debug_menu_active(app) {
        #[cfg(feature = "debug-menu")]
        {
            crate::ui::debug_menu::draw_debug_scenario_lines(app, theme)
        }
        #[cfg(not(feature = "debug-menu"))]
        {
            Vec::new()
        }
    } else if app.main_menu_mode() == MainMenuMode::LoadGame {
        recording_list_lines(app, app.load_entries(), "No in-progress saves.", theme)
    } else if app.main_menu_mode() == MainMenuMode::Replays {
        recording_list_lines(app, app.replay_entries(), "No finished replays.", theme)
    } else if scenarios_menu_active(app) {
        crate::ui::scenario_menu::draw_scenario_lines(app, theme)
    } else {
        let items: Vec<&str> = if app.debug_menu_enabled() {
            vec![
                "Start game",
                "Load game",
                "Replays",
                "Scenarios",
                "Debug",
                "Settings",
                "Exit",
            ]
        } else {
            vec![
                "Start game",
                "Load game",
                "Replays",
                "Scenarios",
                "Settings",
                "Exit",
            ]
        };
        items
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let prefix = if i == app.menu_index() { "> " } else { "  " };
                let style = if i == app.menu_index() {
                    theme.menu_selected_style()
                } else {
                    Style::default().fg(theme.primary)
                };
                Line::from(Span::styled(format!("{prefix}{label}"), style))
            })
            .collect()
    };

    let title = match app.main_menu_mode() {
        MainMenuMode::LoadGame => "Load game",
        MainMenuMode::Replays => "Replays",
        MainMenuMode::Scenarios => "Scenarios",
        #[cfg(feature = "debug-menu")]
        MainMenuMode::Debug => "Debug scenarios",
        MainMenuMode::Root => "Main menu",
    };

    let menu = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title(title),
    );
    frame.render_widget(menu, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::raw("↑/↓ "),
        muted_span("navigate", theme),
        Span::raw("  enter "),
        muted_span("select", theme),
        Span::raw("  h "),
        muted_span("help", theme),
        Span::raw("  ? "),
        muted_span("rules", theme),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

/// Centered settings dialog over the main menu.
pub fn draw_settings_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let popup = popup::open_popup(frame, area, 80, 75);
    draw_settings_content(frame, popup, app, theme);
}

fn draw_settings_content(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let cfg = app.config();
    let field = app.settings_field();
    let theme_label = Theme::resolve(&cfg.theme).label;

    let row = |label: &str, value: &str, selected: bool| -> Line<'static> {
        let prefix = if selected { "> " } else { "  " };
        let style = if selected {
            theme.menu_selected_style()
        } else {
            Style::default().fg(theme.primary)
        };
        Line::from(Span::styled(format!("{prefix}{label}: {value}"), style))
    };

    let lines = vec![
        Line::from("Settings — changes saved when you press Esc"),
        Line::from(""),
        row("Theme", theme_label, field == SettingsField::Theme),
        row(
            "Rules profile",
            cfg.rules_profile.as_str(),
            field == SettingsField::RulesProfile,
        ),
        row(
            "Default CPU difficulty",
            difficulty_label(cfg.default_difficulty),
            field == SettingsField::DefaultDifficulty,
        ),
        row(
            "Preferred human seat",
            crate::app::NewGameSetup::seat_name(cfg.human_seat),
            field == SettingsField::HumanSeat,
        ),
        row(
            "CPU decision delay",
            &crate::timers::label_cpu(cfg.cpu_step_delay_ms),
            field == SettingsField::CpuStepDelay,
        ),
        row(
            "Turn timer",
            &crate::timers::label_turn(cfg.turn_timer_ms),
            field == SettingsField::TurnTimer,
        ),
        row(
            "Call response timer",
            &crate::timers::label_response(cfg.response_timer_ms),
            field == SettingsField::ResponseTimer,
        ),
        Line::from(""),
        Line::from(format!(
            "Config dir: {}",
            crate::config::config_dir().display()
        )),
        Line::from(format!("Config file: {}", app.config_path().display())),
        Line::from(format!(
            "Keybinds: {}",
            app.keybinds_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| match app.keybinds().source() {
                    crate::input::KeybindsSource::Default => "built-in defaults".into(),
                    crate::input::KeybindsSource::File(p) => p.display().to_string(),
                })
        )),
        Line::from(format!("Built-in themes: {}", theme_names().join(", "))),
        Line::from(""),
        Line::from(Span::styled(
            "↑/↓ navigate  enter/space/tab change  esc save & back",
            Style::default().fg(theme.muted),
        )),
    ];

    let widget = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Settings"),
    );
    frame.render_widget(widget, area);
}
