mod config;
pub mod dora;
pub mod flow;
mod profile;
mod recommendations;
mod registry;
pub(crate) mod standard;

#[cfg(test)]
mod tests;

use crate::state::HandState;

pub use config::RulesConfig;
pub use dora::{dora_tile, is_aka_dora, is_hand_dora, matches_indicator_dora};
pub use profile::{RulesProfile, RulesProfileId, WinContext, WinTimingFlags};
pub use recommendations::{PathDecomposition, PathGroup, Recommendation, sort_recommendations};
pub use registry::RulesRegistry;

/// Scored win paths for the active rules profile (planning UI).
pub fn recommendations(
    state: &HandState,
    seat: usize,
    config: &RulesConfig,
    limit: usize,
) -> Vec<Recommendation> {
    let Ok(profile) = RulesRegistry::get(config.profile) else {
        return Vec::new();
    };
    profile.recommendations(state, seat, config, limit)
}
