/// How a seat is controlled during a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlayerSlot {
    Human,
    /// Reserved for Phase 7 AI tiers.
    Cpu,
    /// Reserved for future network play.
    Remote,
}
