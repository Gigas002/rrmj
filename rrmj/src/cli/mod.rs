use std::path::PathBuf;

use clap::Parser;

#[cfg(test)]
mod tests;

/// Raw CLI invocation — parsed only in `main`.
#[derive(Debug, Parser)]
#[command(name = "rrmj", about = "Terminal client for rrmj", version = librrmj::VERSION)]
pub struct Cli {
    /// Path to config.toml (default: $XDG_CONFIG_HOME/rrmj/config.toml).
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Path to keybinds.toml (default: $XDG_CONFIG_HOME/rrmj/keybinds.toml).
    #[arg(long)]
    pub keybinds: Option<PathBuf>,
}
