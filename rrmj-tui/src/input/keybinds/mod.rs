use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyModifiers};

use crate::error::AppError;
use crate::input::key::{BindAction, KeyChord, parse_key_spec};

#[cfg(test)]
mod tests;

/// Full hotkey map loaded from disk or baked-in defaults.
#[derive(Debug, Clone)]
pub struct Keybinds {
    bindings: HashMap<BindAction, KeyChord>,
    reverse: HashMap<KeyChord, BindAction>,
    source: KeybindsSource,
}

#[derive(Debug, Clone)]
pub enum KeybindsSource {
    Default,
    File(PathBuf),
}

impl Keybinds {
    pub fn load(path: Option<&Path>) -> Result<Self, AppError> {
        let path = path
            .map(Path::to_path_buf)
            .unwrap_or_else(crate::config::keybinds_path);

        if path.exists() {
            Self::from_file(&path)
        } else {
            Ok(Self::default_map())
        }
    }

    pub fn default_map() -> Self {
        let mut bindings = HashMap::new();
        for (action, spec) in default_entries() {
            let chord = parse_key_spec(spec).expect("built-in key spec is valid");
            bindings.insert(*action, chord);
        }
        Self::from_bindings(bindings, KeybindsSource::Default)
    }

    pub fn from_file(path: &Path) -> Result<Self, AppError> {
        let text = fs::read_to_string(path).map_err(AppError::Terminal)?;
        let table: toml::Table = toml::from_str(&text).map_err(|err| AppError::Keybinds {
            path: path.to_path_buf(),
            detail: err.to_string(),
        })?;

        let mut bindings = HashMap::new();
        for (section, value) in &table {
            let Some(section_table) = value.as_table() else {
                return Err(AppError::Keybinds {
                    path: path.to_path_buf(),
                    detail: format!("section [{section}] must be a table"),
                });
            };
            for (name, value) in section_table {
                let action = parse_action_name(section, name)?;
                let Some(spec) = value.as_str() else {
                    return Err(AppError::Keybinds {
                        path: path.to_path_buf(),
                        detail: format!("[{section}] {name} must be a string"),
                    });
                };
                let chord = parse_key_spec(spec).map_err(|detail| AppError::Keybinds {
                    path: path.to_path_buf(),
                    detail: format!("[{section}] {name}: {detail}"),
                })?;
                bindings.insert(action, chord);
            }
        }

        // Merge missing entries from defaults so the map stays complete.
        for &(action, spec) in default_entries() {
            bindings
                .entry(action)
                .or_insert_with(|| parse_key_spec(spec).expect("built-in key spec is valid"));
        }

        Ok(Self::from_bindings(
            bindings,
            KeybindsSource::File(path.to_path_buf()),
        ))
    }

    fn from_bindings(bindings: HashMap<BindAction, KeyChord>, source: KeybindsSource) -> Self {
        let reverse = bindings.iter().map(|(a, c)| (*c, *a)).collect();
        Self {
            bindings,
            reverse,
            source,
        }
    }

    pub const fn source(&self) -> &KeybindsSource {
        &self.source
    }

    pub fn chord(&self, action: BindAction) -> KeyChord {
        self.bindings
            .get(&action)
            .copied()
            .unwrap_or_else(|| fallback_chord(action))
    }

    pub fn action_for(&self, event: &crossterm::event::KeyEvent) -> Option<BindAction> {
        let chord = KeyChord::new(event.code, event.modifiers);
        self.reverse.get(&chord).copied()
    }

    /// Whether `event` matches the chord bound to `action` (forward lookup).
    ///
    /// Prefer this over `action_for` when several actions share the same key
    /// (e.g. Enter → select, confirm, and continue).
    pub fn is_bound(&self, event: &crossterm::event::KeyEvent, action: BindAction) -> bool {
        self.chord(action) == KeyChord::new(event.code, event.modifiers)
    }

    pub fn is_any_bound(&self, event: &crossterm::event::KeyEvent, actions: &[BindAction]) -> bool {
        actions.iter().any(|action| self.is_bound(event, *action))
    }

    pub fn entries(&self) -> Vec<(BindAction, KeyChord)> {
        let mut entries: Vec<_> = self.bindings.iter().map(|(a, c)| (*a, *c)).collect();
        entries.sort_by_key(|(action, _)| action_label(*action));
        entries
    }
}

fn parse_action_name(section: &str, name: &str) -> Result<BindAction, AppError> {
    let key = format!("{section}.{name}");
    parse_action_key(&key).ok_or_else(|| AppError::Keybinds {
        path: PathBuf::from("<keybinds>"),
        detail: format!("unknown action '{key}'"),
    })
}

fn parse_action_key(key: &str) -> Option<BindAction> {
    match key {
        "global.quit" => Some(BindAction::Quit),
        "global.help" => Some(BindAction::Help),
        "global.back" => Some(BindAction::Back),
        "global.main_menu" => Some(BindAction::MainMenu),
        "menu.up" => Some(BindAction::MenuUp),
        "menu.down" => Some(BindAction::MenuDown),
        "menu.select" => Some(BindAction::MenuSelect),
        "menu.toggle" => Some(BindAction::MenuToggle),
        "menu.cycle" => Some(BindAction::MenuCycle),
        "table.pass" => Some(BindAction::Pass),
        "table.ron" => Some(BindAction::Ron),
        "table.pon" => Some(BindAction::Pon),
        "table.chi" => Some(BindAction::Chi),
        "table.open_kan" => Some(BindAction::OpenKan),
        "table.kakan" => Some(BindAction::Kakan),
        "table.closed_kan" => Some(BindAction::ClosedKan),
        "table.tsumo" => Some(BindAction::Tsumo),
        "table.riichi" => Some(BindAction::Riichi),
        "table.abort_nine_terminals" => Some(BindAction::AbortNineTerminals),
        "table.discard" => Some(BindAction::Discard),
        "table.tile_prev" => Some(BindAction::TilePrev),
        "table.tile_next" => Some(BindAction::TileNext),
        "table.confirm" => Some(BindAction::Confirm),
        "overlay.continue" => Some(BindAction::Continue),
        "overlay.rules" => Some(BindAction::RulesReference),
        "overlay.scores" => Some(BindAction::Scores),
        "overlay.recommendations" => Some(BindAction::Recommendations),
        _ => None,
    }
}

fn default_entries() -> &'static [(BindAction, &'static str)] {
    &[
        (BindAction::Quit, "q"),
        (BindAction::Help, "h"),
        (BindAction::Back, "esc"),
        (BindAction::MainMenu, "m"),
        (BindAction::MenuUp, "up"),
        (BindAction::MenuDown, "down"),
        (BindAction::MenuSelect, "enter"),
        (BindAction::MenuToggle, "space"),
        (BindAction::MenuCycle, "tab"),
        (BindAction::Pass, "p"),
        (BindAction::Ron, "r"),
        (BindAction::Pon, "o"),
        (BindAction::Chi, "c"),
        (BindAction::OpenKan, "k"),
        (BindAction::Kakan, "u"),
        (BindAction::ClosedKan, "g"),
        (BindAction::Tsumo, "t"),
        (BindAction::Riichi, "i"),
        (BindAction::AbortNineTerminals, "a"),
        (BindAction::Discard, "d"),
        (BindAction::TilePrev, "left"),
        (BindAction::TileNext, "right"),
        (BindAction::Confirm, "enter"),
        (BindAction::Continue, "enter"),
        (BindAction::RulesReference, "?"),
        (BindAction::Scores, "s"),
        (BindAction::Recommendations, "e"),
    ]
}

fn fallback_chord(action: BindAction) -> KeyChord {
    let spec = default_entries()
        .iter()
        .find(|(a, _)| *a == action)
        .map(|(_, s)| *s)
        .unwrap_or("?");
    parse_key_spec(spec).unwrap_or(KeyChord::new(KeyCode::Char('?'), KeyModifiers::empty()))
}

pub fn action_label(action: BindAction) -> &'static str {
    match action {
        BindAction::Quit => "Quit",
        BindAction::Help => "Keybind help",
        BindAction::Back => "Back / cancel",
        BindAction::MainMenu => "Return to main menu (table)",
        BindAction::MenuUp => "Menu up",
        BindAction::MenuDown => "Menu down",
        BindAction::MenuSelect => "Select",
        BindAction::MenuToggle => "Toggle seat type",
        BindAction::MenuCycle => "Cycle difficulty",
        BindAction::Pass => "Pass",
        BindAction::Ron => "Ron",
        BindAction::Pon => "Pon",
        BindAction::Chi => "Chi",
        BindAction::OpenKan => "Open kan",
        BindAction::Kakan => "Added kan (kakan)",
        BindAction::ClosedKan => "Closed kan",
        BindAction::Tsumo => "Tsumo",
        BindAction::Riichi => "Riichi",
        BindAction::AbortNineTerminals => "Abort (nine terminals)",
        BindAction::Discard => "Discard",
        BindAction::TilePrev => "Previous tile",
        BindAction::TileNext => "Next tile",
        BindAction::Confirm => "Confirm",
        BindAction::Continue => "Continue",
        BindAction::RulesReference => "Rules / yaku reference",
        BindAction::Scores => "Scores",
        BindAction::Recommendations => "Win path recommendations",
    }
}

pub fn format_chord(chord: KeyChord) -> String {
    let mut parts = Vec::new();
    if chord.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if chord.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if chord.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }
    let key = chord_label(chord.code);
    parts.push(key.as_str());
    parts.join("+")
}

fn chord_label(code: KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        other => format!("{other:?}"),
    }
}
