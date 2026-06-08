mod config;
mod easy;
mod medium;
mod setup;
mod shanten;

#[cfg(test)]
mod tests;

pub use config::{AiConfig, Difficulty};
pub use easy::EasyAgent;
pub use medium::MediumAgent;
pub use setup::{CpuAgent, MatchSetup, SeatAgent};
