mod board;
#[cfg(feature = "debug-menu")]
mod debug_menu;
#[cfg(feature = "debug-menu")]
mod debug_setup;
mod hand_result;
mod help;
mod load_setup;
mod menu;
mod pause;
mod popup;
mod render;
mod rules;
mod rules_content;
mod scores;
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
    if app.load_setup_open() {
        load_setup::draw_load_setup_popup(frame, area, app, theme);
    }
    #[cfg(feature = "debug-menu")]
    if app.debug_setup_open() {
        debug_setup::draw_debug_setup_popup(frame, area, app, theme);
    }
    if app.settings_open() {
        menu::draw_settings_popup(frame, area, app, theme);
    }
}

fn draw_table_screen(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    table::draw_table(frame, area, app, theme);
    if app.pause_open() {
        pause::draw_pause_popup(frame, area, app.pause_index(), theme);
    } else if app.scores_open() {
        scores::draw_scores_popup(frame, area, app, theme);
    }
    if app.hand_result().is_some() {
        hand_result::draw_hand_result_popup(frame, area, app, theme);
    } else if app.match_summary().is_some() {
        hand_result::draw_match_summary_popup(frame, area, app, theme);
    }
}
