use crate::game::{AbortiveDrawKind, AbortiveTrigger};
use crate::hand::Hand;
use crate::rules::RulesConfig;
use crate::rules::flow::GameFlowPolicy;
use crate::rules::recommendations::Recommendation;
use crate::scoring::{ScoringResult, WinType};
use crate::state::HandState;
use crate::tile::Tile;

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

/// Extra flags for win-timing yaku that are not inferable from hand tiles alone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WinTimingFlags {
    pub is_chankan: bool,
}

/// Context for evaluating a win under a rules profile.
#[derive(Debug, Clone)]
pub struct WinContext<'a> {
    pub state: &'a HandState,
    pub winner: usize,
    pub win_type: WinType,
    pub win_tile: Tile,
    /// Tsumo on the last live-wall tile (haitei raoyue).
    pub is_haitei: bool,
    /// Ron on the last discard with an empty live wall (houtei raoyui).
    pub is_houtei: bool,
    /// Tsumo on a rinshan replacement tile after kan.
    pub is_rinshan: bool,
    /// Ron on a tile added during kakan.
    pub is_chankan: bool,
    /// Takes honba and table riichi sticks (see [`WinContext::with_bonus`]).
    pub collects_bonus: bool,
}

impl<'a> WinContext<'a> {
    pub fn new(
        state: &'a HandState,
        winner: usize,
        win_type: WinType,
        win_tile: Tile,
        timing: WinTimingFlags,
    ) -> Self {
        Self {
            state,
            winner,
            win_type,
            win_tile,
            is_haitei: state.is_haitei_win(win_type),
            is_houtei: state.is_houtei_win(win_type),
            is_rinshan: state.is_rinshan_win(win_type),
            is_chankan: timing.is_chankan,
            collects_bonus: true,
        }
    }

    /// Whether this winner also takes the honba and the riichi sticks on the
    /// table. Only the first winner of a multiple ron does.
    pub fn with_bonus(mut self, collects_bonus: bool) -> Self {
        self.collects_bonus = collects_bonus;
        self
    }

    pub fn hand(&self) -> &Hand {
        self.state.hand(self.winner)
    }

    /// Closed hand: no chi, pon or open kan (closed kans are allowed).
    pub fn is_menzen(&self) -> bool {
        self.hand().is_closed()
    }

    pub fn is_riichi(&self) -> bool {
        self.state.is_riichi(self.winner)
    }

    pub fn is_tenhou(&self) -> bool {
        self.state.is_tenhou_win(self.winner, self.win_type)
    }

    pub fn is_chiihou(&self) -> bool {
        self.state.is_chiihou_win(self.winner, self.win_type)
    }

    pub fn is_renhou(&self) -> bool {
        self.state.is_renhou_win(self.winner, self.win_type)
    }
}

/// Ruleset implementation boundary (yaku, scoring, draw policies).
pub trait RulesProfile: Send + Sync {
    fn id(&self) -> RulesProfileId;

    fn can_win(&self, ctx: &WinContext<'_>, config: &RulesConfig) -> bool;

    fn is_tenpai(&self, hand: &Hand, config: &RulesConfig) -> bool;

    /// Whether discarding this tile keeps the hand tenpai (required for riichi declaration).
    fn is_riichi_discard(&self, hand: &Hand, discard: Tile, config: &RulesConfig) -> bool;

    /// Whether a seat in riichi may declare a closed kan of `kan_tile` after drawing `drawn`.
    fn riichi_kan_allowed(
        &self,
        hand: &Hand,
        drawn: Tile,
        kan_tile: Tile,
        config: &RulesConfig,
    ) -> bool;

    fn score_win(&self, ctx: &WinContext<'_>, config: &RulesConfig) -> ScoringResult;

    /// Scored win paths for planning UI (sorted by expected points, then shanten).
    fn recommendations(
        &self,
        state: &HandState,
        seat: usize,
        config: &RulesConfig,
        limit: usize,
    ) -> Vec<Recommendation>;

    fn score_exhaustive_draw(&self, state: &HandState, config: &RulesConfig) -> [i32; 4];

    fn detect_abortive(
        &self,
        state: &HandState,
        config: &RulesConfig,
        trigger: AbortiveTrigger,
    ) -> Option<AbortiveDrawKind>;

    fn game_flow(&self) -> &dyn GameFlowPolicy;
}
