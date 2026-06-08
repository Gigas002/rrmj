use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

const LOGO: &str = r"
  ██████  ██████  ███    ███
  ██   ██ ██   ██ ████  ████
  ██████  ██████  ██ ████ ██
  ██   ██ ██   ██ ██  ██  ██
  ██   ██ ██   ██ ██      ██
";

pub fn draw_main_menu(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(area);

    let logo = Paragraph::new(LOGO)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("rrmj"));
    frame.render_widget(logo, chunks[0]);

    let items = ["Start game", "Settings", "Exit"];
    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let prefix = if i == app.menu_index() { "> " } else { "  " };
            let style = if i == app.menu_index() {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(Span::styled(format!("{prefix}{label}"), style))
        })
        .collect();

    let menu = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Main menu"));
    frame.render_widget(menu, chunks[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::raw("↑/↓ "),
        Span::styled("navigate", Style::default().fg(Color::DarkGray)),
        Span::raw("  enter "),
        Span::styled("select", Style::default().fg(Color::DarkGray)),
        Span::raw("  h "),
        Span::styled("help", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

pub fn draw_settings(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let lines = vec![
        Line::from("Settings (in-memory until config.toml in v0.1)"),
        Line::from(""),
        Line::from(format!(
            "Default CPU difficulty: {}",
            crate::app::difficulty_label(app.default_difficulty())
        )),
        Line::from(format!(
            "Preferred human seat: {}",
            crate::app::NewGameSetup::seat_name(app.human_seat())
        )),
        Line::from(""),
        Line::from(format!(
            "Keybinds: {}",
            app.keybinds_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| match app.keybinds().source() {
                    crate::input::KeybindsSource::Default => "built-in defaults".into(),
                    crate::input::KeybindsSource::File(p) => p.display().to_string(),
                })
        )),
        Line::from(format!(
            "Config dir: {}",
            crate::config::config_dir().display()
        )),
        Line::from(""),
        Line::from("tab — cycle default difficulty"),
        Line::from("enter — cycle human seat"),
        Line::from("esc — back"),
    ];

    let widget =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Settings"));
    frame.render_widget(widget, area);
}
