use crate::game::{AbortiveDrawKind, AbortiveTrigger};
use crate::hand::Hand;
use crate::rules::RulesConfig;
use crate::rules::RulesProfileId;
use crate::rules::flow::MatchFlowPolicy;
use crate::scoring::{ScoringResult, WinType};
use crate::state::HandState;
use crate::tile::Tile;

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
        }
    }

    pub fn hand(&self) -> &Hand {
        self.state.hand(self.winner)
    }

    pub fn is_menzen(&self) -> bool {
        self.hand().melds().is_empty()
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

    fn score_win(&self, ctx: &WinContext<'_>, config: &RulesConfig) -> ScoringResult;

    fn score_exhaustive_draw(&self, state: &HandState, config: &RulesConfig) -> [i32; 4];

    fn detect_abortive(
        &self,
        state: &HandState,
        config: &RulesConfig,
        trigger: AbortiveTrigger,
    ) -> Option<AbortiveDrawKind>;

    fn match_flow(&self) -> &dyn MatchFlowPolicy;
}
