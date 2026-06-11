/// Identifies a ruleset implementation (yaku table, scoring, match flow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RulesProfileId {
    Standard,
}

impl RulesProfileId {
    pub const ALL: [Self; 1] = [Self::Standard];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
        }
    }

    pub fn parse(name: &str) -> Result<Self, String> {
        match name.to_ascii_lowercase().as_str() {
            "standard" => Ok(Self::Standard),
            other => Err(format!(
                "unknown rules profile '{other}' (expected standard)"
            )),
        }
    }

    pub fn next(self) -> Self {
        let all = Self::ALL;
        let idx = all.iter().position(|p| *p == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
}
