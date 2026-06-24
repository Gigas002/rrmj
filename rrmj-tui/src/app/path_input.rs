use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Single-line filesystem path entry overlay.
#[derive(Debug, Clone)]
pub struct PathInputDialog {
    path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathInputAction {
    Continue,
    Cancel,
    Confirm(String),
}

impl PathInputDialog {
    pub fn new(default_path: impl AsRef<Path>) -> Self {
        Self {
            path: default_path.as_ref().display().to_string(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        is_activate: bool,
        is_back: bool,
    ) -> PathInputAction {
        if is_back {
            return PathInputAction::Cancel;
        }
        if is_activate {
            return PathInputAction::Confirm(self.path.trim().to_string());
        }

        match key.code {
            KeyCode::Backspace => {
                self.path.pop();
            }
            KeyCode::Char(c)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.path.push(c);
            }
            _ => {}
        }
        PathInputAction::Continue
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

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
}
