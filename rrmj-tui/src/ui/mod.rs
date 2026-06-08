mod board;
mod hand_result;
mod help;
mod menu;
mod popup;
mod render;
mod rules;
mod setup;
mod table;
mod widgets;

pub use rules::rules_line_count;

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::{App, Screen};
use crate::theme::Theme;

pub fn draw(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();
    match app.screen() {
        Screen::MainMenu => draw_main_menu_screen(frame, area, app, theme),
        Screen::Table => draw_table_screen(frame, area, app, theme),
    }

    if app.help_open() {
        help::draw_help_popup(frame, area, app, theme);
    }
    if app.rules_open() {
        rules::draw_rules_popup(frame, area, app, theme);
    }
}

fn draw_main_menu_screen(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    menu::draw_main_menu(frame, area, app, theme);
    if app.setup_open() {
        setup::draw_setup_popup(frame, area, app, theme);
    }
    if app.settings_open() {
        menu::draw_settings_popup(frame, area, app, theme);
    }
}

fn draw_table_screen(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    table::draw_table(frame, area, app, theme);
    if app.hand_result().is_some() {
        hand_result::draw_hand_result_popup(frame, area, app, theme);
    } else if app.match_summary().is_some() {
        hand_result::draw_match_summary_popup(frame, area, app, theme);
    }
}
