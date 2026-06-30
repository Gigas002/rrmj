use clap::Parser;

use super::Cli;

#[test]
fn parses_without_args() {
    let cli = Cli::try_parse_from(["rrmj-tui"]).unwrap();
    assert!(cli.config.is_none());
    assert!(cli.keybinds.is_none());
}

#[test]
fn parses_config_and_keybinds_paths() {
    let cli = Cli::try_parse_from([
        "rrmj-tui",
        "--config",
        "/tmp/config.toml",
        "--keybinds",
        "/tmp/keybinds.toml",
    ])
    .unwrap();
    assert_eq!(
        cli.config.as_deref().unwrap().to_str(),
        Some("/tmp/config.toml")
    );
    assert_eq!(
        cli.keybinds.as_deref().unwrap().to_str(),
        Some("/tmp/keybinds.toml")
    );
}
