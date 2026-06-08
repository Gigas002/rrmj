use super::RulesProfileId;
use crate::game::MatchLength;

/// Tunable parameters within a [`RulesProfileId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesConfig {
    pub profile: RulesProfileId,
    pub starting_points: i32,
    pub aka_dora: bool,
    pub kiriage: bool,
    pub match_length: MatchLength,
    /// When set, the match ends as soon as any seat reaches this score.
    pub target_score: Option<i32>,
    pub abortive_nine_terminals: bool,
    pub abortive_four_winds: bool,
    pub abortive_four_kongs: bool,
    pub abortive_four_riichis: bool,
}

impl RulesConfig {
    /// Default parameters for standard Japanese riichi.
    pub fn standard() -> Self {
        Self::default_for(RulesProfileId::Standard)
    }

    pub fn default_for(profile: RulesProfileId) -> Self {
        match profile {
            RulesProfileId::Standard => Self {
                profile,
                starting_points: 25_000,
                aka_dora: true,
                kiriage: false,
                match_length: MatchLength::Hanchan,
                target_score: None,
                abortive_nine_terminals: true,
                abortive_four_winds: true,
                abortive_four_kongs: true,
                abortive_four_riichis: true,
            },
        }
    }
}
