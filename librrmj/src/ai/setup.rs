use crate::action::Action;
use crate::agent::{Agent, PlayerSlot, PlayerView};
use crate::ai::config::{AiConfig, Difficulty};
use crate::ai::easy::EasyAgent;
use crate::ai::hard::HardAgent;
use crate::ai::medium::MediumAgent;

/// CPU opponent implementing a difficulty tier.
#[derive(Debug)]
pub enum CpuAgent {
    Easy(EasyAgent),
    Medium(MediumAgent),
    Hard(HardAgent),
}

impl CpuAgent {
    pub fn new(config: AiConfig, seat: usize, match_seed: u64) -> Self {
        let seed = config
            .personality_seed
            .unwrap_or(match_seed)
            .wrapping_add(seat as u64)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15);
        match config.difficulty {
            Difficulty::Easy => Self::Easy(EasyAgent::new(seed)),
            Difficulty::Medium => Self::Medium(MediumAgent::new(seed)),
            Difficulty::Hard => Self::Hard(HardAgent::new(seed)),
        }
    }
}

impl Agent for CpuAgent {
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action {
        match self {
            Self::Easy(agent) => agent.decide(view, legal),
            Self::Medium(agent) => agent.decide(view, legal),
            Self::Hard(agent) => agent.decide(view, legal),
        }
    }
}

/// Seat agent used when wiring a match from [`MatchSetup`].
#[derive(Debug)]
pub enum SeatAgent {
    Cpu(Box<CpuAgent>),
    /// Placeholder until a client supplies human input.
    HumanPending,
}

impl Agent for SeatAgent {
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action {
        match self {
            Self::Cpu(agent) => agent.as_mut().decide(view, legal),
            Self::HumanPending => legal
                .iter()
                .find(|action| matches!(action, Action::Pass))
                .copied()
                .unwrap_or(legal[0]),
        }
    }
}

/// Per-seat assignment for building agents alongside a [`crate::game::Match`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchSetup {
    pub slots: [PlayerSlot; 4],
    pub default_ai: AiConfig,
    pub seat_ai: [Option<AiConfig>; 4],
}

impl MatchSetup {
    pub fn all_cpu(config: AiConfig) -> Self {
        Self {
            slots: [PlayerSlot::Cpu; 4],
            default_ai: config,
            seat_ai: [None; 4],
        }
    }

    pub fn all_easy(match_seed: u64) -> Self {
        Self::all_cpu(AiConfig::easy(match_seed))
    }

    pub fn all_medium(match_seed: u64) -> Self {
        Self::all_cpu(AiConfig::medium(match_seed))
    }

    pub fn all_hard(match_seed: u64) -> Self {
        Self::all_cpu(AiConfig::hard(match_seed))
    }

    /// Seats 0 and 2 use hard AI; seats 1 and 3 use medium (for benchmarks).
    pub fn hard_vs_medium(match_seed: u64) -> Self {
        Self {
            slots: [PlayerSlot::Cpu; 4],
            default_ai: AiConfig::medium(match_seed),
            seat_ai: [
                Some(AiConfig::hard(match_seed)),
                None,
                Some(AiConfig::hard(match_seed.wrapping_add(1))),
                None,
            ],
        }
    }

    pub fn ai_config_for(&self, seat: usize) -> Option<AiConfig> {
        match self.slots[seat] {
            PlayerSlot::Cpu => Some(self.seat_ai[seat].unwrap_or(self.default_ai)),
            _ => None,
        }
    }

    pub fn build_agents(&self, match_seed: u64) -> [SeatAgent; 4] {
        std::array::from_fn(|seat| match self.slots[seat] {
            PlayerSlot::Cpu => {
                let config = self.ai_config_for(seat).expect("cpu seat has ai config");
                SeatAgent::Cpu(Box::new(CpuAgent::new(config, seat, match_seed)))
            }
            PlayerSlot::Human | PlayerSlot::Remote => SeatAgent::HumanPending,
        })
    }
}
