mod common;
mod config;
mod defense;
mod easy;
mod efficiency;
mod hard;
mod medium;
mod setup;
mod shanten;

#[cfg(test)]
mod tests;

pub use config::{AiConfig, Difficulty};
pub use easy::EasyAgent;
pub use hard::HardAgent;
pub use medium::MediumAgent;
pub use setup::{CpuAgent, MatchSetup, SeatAgent};
