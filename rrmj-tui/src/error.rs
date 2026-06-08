use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("librrmj: {0}")]
    Engine(#[from] librrmj::Error),
    #[error("terminal: {0}")]
    Terminal(#[from] std::io::Error),
    #[error("keybinds at {path}: {detail}")]
    Keybinds { path: PathBuf, detail: String },
}
