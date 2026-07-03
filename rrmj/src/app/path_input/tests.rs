use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{PathInputAction, PathInputDialog};

fn char_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty())
}

#[test]
fn edits_path_and_confirms() {
    let mut dialog = PathInputDialog::new(PathBuf::from("/tmp/a.json"));
    assert_eq!(
        dialog.handle_key(char_key('x'), false, false),
        PathInputAction::Continue
    );
    assert!(dialog.path().ends_with('x'));
    assert_eq!(
        dialog.handle_key(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
            true,
            false
        ),
        PathInputAction::Confirm("/tmp/a.jsonx".into())
    );
}

#[test]
fn backspace_and_cancel() {
    let mut dialog = PathInputDialog::new("x");
    assert_eq!(
        dialog.handle_key(
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()),
            false,
            false
        ),
        PathInputAction::Continue
    );
    assert_eq!(dialog.path(), "");
    assert_eq!(
        dialog.handle_key(
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
            false,
            true
        ),
        PathInputAction::Cancel
    );
}
