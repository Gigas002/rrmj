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

pub use action::{Action, KanIntent};
pub use agent::{
    Agent, FnAgent, PendingCall, PlayerSlot, PlayerView, SeatView, TurnContext, TurnFocus,
};
#[cfg(feature = "ai")]
pub use ai::{
    AiConfig, CpuAgent, Difficulty, EasyAgent, GameSetup, HardAgent, MediumAgent, SeatAgent,
};
pub use error::Error;
pub use event::Event;
pub use game::{
    AbortiveDrawKind, Game, GameLength, GamePhase, HandOutcome, RoundWind, StepResult,
};
pub use hand::{Concealed, Hand, KanForm, Meld, MeldKind};
#[cfg(feature = "serde")]
pub use replay::{
    FORMAT_VERSION, GameRecording, GameStatus, HandSnapshot, PlayerSetup, RecordingMeta,
    RecordingPlayer,
};
pub use replay::{GameSnapshot, Replay};
pub use rules::{
    Recommendation, RulesConfig, RulesProfile, RulesProfileId, RulesRegistry, WinContext,
    WinTimingFlags, recommendations, sort_recommendations,
};
pub use scoring::{ScoringResult, WinType, Yaku};
pub use state::{HandPhase, HandState, SEAT_COUNT};
pub use tile::{Tile, TileIdentity, TileKind, standard_set};
pub use wall::{DealResult, Wall};
