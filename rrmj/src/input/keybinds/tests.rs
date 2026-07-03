use std::io::Write;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::NamedTempFile;

use crate::input::key::{BindAction, KeyChord, parse_key_spec};
use crate::input::keybinds::Keybinds;

#[test]
fn default_map_is_complete() {
    let binds = Keybinds::default_map();
    assert_eq!(
        binds.chord(BindAction::Help),
        KeyChord::new(KeyCode::Char('h'), KeyModifiers::empty())
    );
    assert!(binds.entries().len() >= 20);
    assert_eq!(
        binds.chord(BindAction::MainMenu),
        KeyChord::new(KeyCode::Char('m'), KeyModifiers::empty())
    );
}

#[test]
fn parse_key_spec_supports_named_keys() {
    let chord = parse_key_spec("enter").unwrap();
    assert_eq!(chord.code, KeyCode::Enter);
    let chord = parse_key_spec("ctrl+q").unwrap();
    assert_eq!(chord.code, KeyCode::Char('q'));
    assert!(chord.modifiers.contains(KeyModifiers::CONTROL));
}

#[test]
fn unknown_action_in_file_errors() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
[global]
unknown_action = "x"
"#
    )
    .unwrap();
    let err = Keybinds::from_file(file.path()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown action"));
}

#[test]
fn enter_binds_multiple_activate_actions() {
    let binds = Keybinds::default_map();
    let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    assert!(binds.action_for(&enter).is_some());
    assert!(binds.is_bound(&enter, BindAction::MenuSelect));
    assert!(binds.is_bound(&enter, BindAction::Confirm));
    assert!(binds.is_bound(&enter, BindAction::Continue));
}

#[test]
fn file_overrides_defaults() {
    let mut file = NamedTempFile::new().unwrap();
    write!(
        file,
        r#"
[global]
help = "?"
"#
    )
    .unwrap();
    let binds = Keybinds::from_file(file.path()).unwrap();
    assert_eq!(
        binds.chord(BindAction::Help),
        KeyChord::new(KeyCode::Char('?'), KeyModifiers::empty())
    );
    assert_eq!(
        binds.chord(BindAction::Quit),
        KeyChord::new(KeyCode::Char('q'), KeyModifiers::empty())
    );
}
