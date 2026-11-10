use librrmj::ai::Difficulty;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
        .expect("key handled");
}

fn app_in(dir: &std::path::Path) -> App {
    let settings = Settings {
        recordings_dir: Some(dir.join("recordings")),
        scenarios_dir: Some(dir.join("scenarios")),
        ..Settings::default()
    };
    App::new(settings)
}

/// A match with the human in seat 0 and three CPUs, nothing played yet.
fn app_with_match(dir: &std::path::Path) -> App {
    let mut app = app_in(dir);
    app.setup = Some(NewGameSetup::new(Difficulty::Easy, 0, 0, 0, 0));
    app.start_match().expect("match starts");
    app
}

#[test]
fn leaving_an_unsaved_match_needs_a_second_press() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = app_with_match(dir.path());

    press(&mut app, KeyCode::Char('m'));
    assert_eq!(app.screen, Screen::Table, "first press only warns");
    assert!(app.status.contains("Unsaved"), "{}", app.status);
    press(&mut app, KeyCode::Char('m'));
    assert_eq!(app.screen, Screen::MainMenu);

    let mut app = app_with_match(dir.path());
    press(&mut app, KeyCode::Char('q'));
    assert!(!app.quit);
    press(&mut app, KeyCode::Char('q'));
    assert!(app.quit);
}

#[test]
fn another_key_cancels_a_pending_leave() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = app_with_match(dir.path());

    press(&mut app, KeyCode::Char('m'));
    press(&mut app, KeyCode::Char('h'));
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('m'));
    assert_eq!(app.screen, Screen::Table, "the leave had to be armed again");
}

#[test]
fn a_saved_match_leaves_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = app_with_match(dir.path());

    // Pause menu → Export save → accept the default path.
    press(&mut app, KeyCode::Esc);
    while app.pause_index != PauseItem::ExportSave {
        press(&mut app, KeyCode::Down);
    }
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    assert!(app.status.starts_with("Saved to"), "{}", app.status);

    press(&mut app, KeyCode::Char('m'));
    assert_eq!(app.screen, Screen::MainMenu);
}

#[test]
fn a_corrupt_save_does_not_break_the_menus() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = app_in(dir.path());
    let recordings = dir.path().join("recordings");
    std::fs::create_dir_all(&recordings).unwrap();
    std::fs::write(
        recordings.join("match-1.json"),
        "{\"format_version\": 3, \"ti",
    )
    .unwrap();

    app.open_load_game_menu().expect("load menu opens");
    assert!(app.load_entries.is_empty());
    app.open_replays_menu().expect("replays menu opens");
    assert!(app.replay_entries.is_empty());
}

#[test]
fn errors_are_reported_not_fatal() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = app_in(dir.path());
    app.report_error(Err(AppError::Config {
        path: PathBuf::from("x"),
        detail: "boom".into(),
    }));
    assert!(app.status.starts_with("Error:"), "{}", app.status);
    assert!(!app.quit);
}

/// Opens the bundled scenario `name` through the Scenarios menu with keys.
fn launch_scenario(dir: &std::path::Path, name: &str) -> App {
    let mut app = app_in(dir);
    app.settings.scenarios_dir = Some(crate::scenarios::bundled_debug_scenarios_dir());
    app.open_scenarios_menu().expect("scenarios menu");
    let index = app
        .filtered_scenario_entries()
        .iter()
        .position(|entry| entry.path.file_stem().is_some_and(|stem| stem == name))
        .expect("scenario listed");
    app.menu_index = index;
    press(&mut app, KeyCode::Enter);
    let setup = app.scenario_setup.as_mut().expect("seat picker open");
    setup.selected = LoadSetupField::Confirm;
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.screen, Screen::Table, "{}", app.status);
    app
}

#[test]
fn a_scenario_saved_after_its_hand_ended_moves_on() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = launch_scenario(dir.path(), "four_winds_abortive");
    let result = app.hand_result.clone().expect("the ended hand is shown");
    assert_eq!(result.title, "Abortive draw");

    press(&mut app, KeyCode::Enter);
    assert!(app.hand_result.is_none());
    let game = app.active_game.as_ref().unwrap();
    assert!(
        game.pending_seat().is_some(),
        "the next hand is dealt and someone must act"
    );
}

#[test]
fn a_finished_match_scenario_shows_the_summary() {
    let dir = tempfile::tempdir().unwrap();
    let app = launch_scenario(dir.path(), "match_finished");
    assert!(app.game_summary.is_some());
}

#[test]
fn keys_in_the_scenario_seat_picker_stay_there() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = app_in(dir.path());
    app.settings.scenarios_dir = Some(crate::scenarios::bundled_debug_scenarios_dir());
    app.open_scenarios_menu().unwrap();
    press(&mut app, KeyCode::Enter);
    assert!(app.scenario_setup.is_some());
    let menu_index = app.menu_index;
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.menu_index, menu_index, "the menu below did not move");
}
