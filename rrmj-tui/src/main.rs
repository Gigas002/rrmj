mod app;
mod config;
mod error;
mod input;
mod logger;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::app::App;
use crate::error::AppError;
use crate::input::Keybinds;

#[derive(Parser)]
#[command(name = "rrmj-tui", about = "Terminal client for rrmj", version = librrmj::VERSION)]
struct Cli {
    /// Path to keybinds.toml (default: $XDG_CONFIG_HOME/rrmj/keybinds.toml).
    #[arg(long)]
    keybinds: Option<PathBuf>,
}

fn main() -> ExitCode {
    logger::init();
    let cli = Cli::parse();

    let keybinds_path = cli.keybinds.clone();
    let keybinds = match Keybinds::load(cli.keybinds.as_deref()) {
        Ok(binds) => binds,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(1);
        }
    };

    tracing::debug!(rules = ?librrmj::RulesConfig::standard(), "initialized");

    let mut app = App::new(keybinds, keybinds_path);
    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(AppError::Terminal(err)) if err.kind() == std::io::ErrorKind::Interrupted => {
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
