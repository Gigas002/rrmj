mod hand_result;
mod help;
mod menu;
mod setup;
mod table;
mod widgets;

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    match app.screen() {
        Screen::MainMenu => menu::draw_main_menu(frame, area, app),
        Screen::NewGameSetup => setup::draw_setup(frame, area, app),
        Screen::Settings => menu::draw_settings(frame, area, app),
        Screen::Table => draw_table_screen(frame, area, app),
        Screen::HandResult => hand_result::draw_hand_result(frame, area, app),
        Screen::MatchSummary => hand_result::draw_match_summary(frame, area, app),
    }

    if app.help_open() {
        help::draw_help(frame, area, app);
    }
}

fn draw_table_screen(frame: &mut Frame, area: Rect, app: &App) {
    if app.hand_result().is_some() {
        hand_result::draw_hand_result(frame, area, app);
    } else {
        table::draw_table(frame, area, app);
    }
}
