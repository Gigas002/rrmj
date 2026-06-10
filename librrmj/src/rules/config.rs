use super::RulesProfileId;
use crate::game::MatchLength;

#[cfg(feature = "serde")]
fn default_double_ron() -> bool {
    true
}

/// Tunable parameters within a [`RulesProfileId`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Two players may win on the same discard.
    #[cfg_attr(feature = "serde", serde(default = "default_double_ron"))]
    pub double_ron: bool,
    /// Three players may win on the same discard (requires `double_ron`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub triple_ron: bool,
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
                double_ron: true,
                triple_ron: false,
            },
        }
    }

    pub fn max_rons(&self) -> usize {
        if self.triple_ron {
            3
        } else if self.double_ron {
            2
        } else {
            1
        }
    }
}
