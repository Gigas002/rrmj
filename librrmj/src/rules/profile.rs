/// Identifies a ruleset implementation (yaku table, scoring, match flow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RulesProfileId {
    Standard,
}

impl RulesProfileId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
        }
    }
}
