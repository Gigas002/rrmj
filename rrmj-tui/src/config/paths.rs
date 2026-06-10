use std::path::PathBuf;

/// XDG-style config directory: `$XDG_CONFIG_HOME/rrmj` or `~/.config/rrmj`.
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("rrmj");
    }
    dirs_home().join(".config").join("rrmj")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn keybinds_path() -> PathBuf {
    config_dir().join("keybinds.toml")
}

/// `$XDG_DATA_HOME/rrmj` or `~/.local/share/rrmj`.
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(dir).join("rrmj");
    }
    dirs_home().join(".local").join("share").join("rrmj")
}

/// All match recordings (in-progress and finished) live in one directory.
pub fn recordings_dir() -> PathBuf {
    data_dir().join("recordings")
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
