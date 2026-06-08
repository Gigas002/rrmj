use super::RulesProfileId;

/// Tunable parameters within a [`RulesProfileId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesConfig {
    pub profile: RulesProfileId,
    pub starting_points: i32,
    pub aka_dora: bool,
    pub kiriage: bool,
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
            },
        }
    }
}
