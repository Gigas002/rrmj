//! Riichi mahjong rules engine.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod error;
pub mod rules;

pub use error::Error;
pub use rules::{RulesConfig, RulesProfileId};
