/// CPU difficulty tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

/// Parameters for a CPU seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AiConfig {
    pub difficulty: Difficulty,
    /// Per-seat RNG seed; defaults to the match seed when unset at build time.
    pub personality_seed: Option<u64>,
}

impl AiConfig {
    pub const fn new(difficulty: Difficulty, personality_seed: u64) -> Self {
        Self {
            difficulty,
            personality_seed: Some(personality_seed),
        }
    }

    pub const fn easy(seed: u64) -> Self {
        Self::new(Difficulty::Easy, seed)
    }

    pub const fn medium(seed: u64) -> Self {
        Self::new(Difficulty::Medium, seed)
    }

    pub const fn hard(seed: u64) -> Self {
        Self::new(Difficulty::Hard, seed)
    }
}
