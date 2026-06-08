//! Riichi mahjong rules engine.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod error;
pub mod hand;
pub mod rules;
pub mod tile;
pub mod wall;

pub use error::Error;
pub use hand::{Concealed, Hand, Meld, MeldKind};
pub use rules::{RulesConfig, RulesProfileId};
pub use tile::{Tile, TileKind, standard_set};
pub use wall::{DealResult, Wall};
