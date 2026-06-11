mod config;
pub mod flow;
mod profile;
mod profile_trait;
mod registry;
pub(crate) mod standard;
mod win_path;

#[cfg(test)]
mod tests;

pub use config::RulesConfig;
pub use profile::RulesProfileId;
pub use profile_trait::{RulesProfile, WinContext, WinTimingFlags};
pub use registry::RulesRegistry;
pub use standard::candidate_win_paths;
pub use win_path::{WinPathCandidate, sort_win_paths};
