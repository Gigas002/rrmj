mod key;
mod keybinds;

pub use key::{BindAction, normalize_key_event};
pub use keybinds::{Keybinds, action_label, format_chord};
