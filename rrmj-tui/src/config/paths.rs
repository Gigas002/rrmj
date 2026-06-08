use std::path::PathBuf;

/// XDG-style config directory: `$XDG_CONFIG_HOME/rrmj` or `~/.config/rrmj`.
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("rrmj");
    }
    dirs_home().join(".config").join("rrmj")
}

pub fn keybinds_path() -> PathBuf {
    config_dir().join("keybinds.toml")
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
