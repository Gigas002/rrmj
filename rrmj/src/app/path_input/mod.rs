use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[cfg(test)]
mod tests;

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
