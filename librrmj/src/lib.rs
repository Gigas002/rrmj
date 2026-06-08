//! Riichi mahjong rules engine.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod action;
pub mod error;
pub mod event;
pub mod hand;
pub mod rules;
pub mod state;
pub mod tile;
pub mod wall;

pub use action::Action;
pub use error::Error;
pub use event::Event;
pub use hand::{Concealed, Hand, Meld, MeldKind};
pub use rules::{RulesConfig, RulesProfileId};
pub use state::{HandPhase, HandState, SEAT_COUNT};
pub use tile::{standard_set, Tile, TileKind};
pub use wall::{DealResult, Wall};
