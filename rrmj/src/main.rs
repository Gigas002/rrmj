mod app;
mod cli;
mod config;
mod error;
mod input;
mod logger;
mod save;
mod scenarios;
mod settings;
mod theme;
mod ui;
mod utils;

use std::process::ExitCode;

use clap::Parser;

use crate::app::App;
use crate::cli::Cli;
use crate::error::AppError;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let settings = match settings::Settings::resolve(&cli) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(1);
        }
    };

    logger::init(&settings);
    tracing::debug!(rules = ?settings.rules_config(), "initialized");

    let mut app = App::new(settings);
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
