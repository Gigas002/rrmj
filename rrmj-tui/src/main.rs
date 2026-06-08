mod logger;

use std::process::ExitCode;

use clap::Parser;

#[derive(Parser)]
#[command(name = "rrmj-tui", about = "Terminal client for rrmj", version = librrmj::VERSION)]
struct Cli {}

fn main() -> ExitCode {
    logger::init();
    let _cli = Cli::parse();
    println!("rrmj-tui {}", librrmj::VERSION);
    tracing::debug!(rules = ?librrmj::RulesConfig::standard(), "initialized");
    ExitCode::SUCCESS
}
