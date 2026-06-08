//! Riichi mahjong rules engine.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod action;
pub mod agent;
#[cfg(feature = "ai")]
pub mod ai;
pub mod error;
pub mod event;
pub mod game;
pub mod hand;
pub mod replay;
pub mod rules;
pub mod scoring;
pub mod state;
pub mod tile;
pub mod wall;

pub use action::Action;
pub use agent::{Agent, FnAgent, PendingCall, PlayerSlot, PlayerView, SeatView};
#[cfg(feature = "ai")]
pub use ai::{AiConfig, CpuAgent, Difficulty, EasyAgent, MatchSetup, MediumAgent, SeatAgent};
pub use error::Error;
pub use event::Event;
pub use game::{
    AbortiveDrawKind, HandOutcome, Match, MatchLength, MatchPhase, RoundWind, StepResult,
};
pub use hand::{Concealed, Hand, Meld, MeldKind};
pub use replay::{MatchSnapshot, Replay};
pub use rules::{RulesConfig, RulesProfile, RulesProfileId, RulesRegistry, WinContext};
pub use scoring::{ScoringResult, WinType, Yaku};
pub use state::{HandPhase, HandState, SEAT_COUNT};
pub use tile::{Tile, TileIdentity, TileKind, standard_set};
pub use wall::{DealResult, Wall};
