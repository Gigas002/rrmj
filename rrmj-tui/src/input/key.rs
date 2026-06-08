use crossterm::event::{KeyCode, KeyModifiers};

/// Logical binding target used by the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindAction {
    // global
    Quit,
    Help,
    Back,
    // menu / setup
    MenuUp,
    MenuDown,
    MenuSelect,
    MenuToggle,
    MenuCycle,
    // table
    Pass,
    Ron,
    Pon,
    Chi,
    OpenKan,
    ClosedKan,
    Tsumo,
    Riichi,
    AbortNineTerminals,
    Discard,
    TilePrev,
    TileNext,
    Confirm,
    Continue,
}

/// A parsed key chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyChord {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

pub fn parse_key_spec(spec: &str) -> Result<KeyChord, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err("empty key spec".into());
    }

    let (modifiers, key_part) = parse_modifiers(spec);
    let code = parse_key_code(key_part)?;
    Ok(KeyChord::new(code, modifiers))
}

fn parse_modifiers(spec: &str) -> (KeyModifiers, &str) {
    let mut mods = KeyModifiers::empty();
    let mut rest = spec;
    while let Some((head, tail)) = rest.split_once('+') {
        match head.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods |= KeyModifiers::CONTROL,
            "alt" => mods |= KeyModifiers::ALT,
            "shift" => mods |= KeyModifiers::SHIFT,
            other => {
                return (
                    mods,
                    if mods.is_empty() {
                        spec
                    } else {
                        &spec[other.len() + 1..]
                    },
                );
            }
        }
        rest = tail;
    }
    (mods, rest)
}

fn parse_key_code(spec: &str) -> Result<KeyCode, String> {
    match spec.to_ascii_lowercase().as_str() {
        "enter" | "return" => Ok(KeyCode::Enter),
        "esc" | "escape" => Ok(KeyCode::Esc),
        "space" => Ok(KeyCode::Char(' ')),
        "tab" => Ok(KeyCode::Tab),
        "backspace" => Ok(KeyCode::Backspace),
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "pageup" => Ok(KeyCode::PageUp),
        "pagedown" => Ok(KeyCode::PageDown),
        "home" => Ok(KeyCode::Home),
        "end" => Ok(KeyCode::End),
        s if s.len() == 1 => {
            let ch = s.chars().next().expect("single char");
            Ok(KeyCode::Char(ch))
        }
        other => Err(format!("unknown key '{other}'")),
    }
}
