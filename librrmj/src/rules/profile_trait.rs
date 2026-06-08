use crate::game::{AbortiveDrawKind, AbortiveTrigger};
use crate::hand::Hand;
use crate::rules::RulesConfig;
use crate::rules::RulesProfileId;
use crate::rules::flow::MatchFlowPolicy;
use crate::scoring::{ScoringResult, WinType};
use crate::state::HandState;
use crate::tile::Tile;

/// Context for evaluating a win under a rules profile.
#[derive(Debug, Clone)]
pub struct WinContext<'a> {
    pub state: &'a HandState,
    pub winner: usize,
    pub win_type: WinType,
    pub win_tile: Tile,
}

impl<'a> WinContext<'a> {
    pub fn hand(&self) -> &Hand {
        self.state.hand(self.winner)
    }

    pub fn is_menzen(&self) -> bool {
        self.hand().melds().is_empty()
    }

    pub fn is_riichi(&self) -> bool {
        self.state.is_riichi(self.winner)
    }
}

/// Ruleset implementation boundary (yaku, scoring, draw policies).
pub trait RulesProfile: Send + Sync {
    fn id(&self) -> RulesProfileId;

    fn can_win(&self, ctx: &WinContext<'_>, config: &RulesConfig) -> bool;

    fn is_tenpai(&self, hand: &Hand, config: &RulesConfig) -> bool;

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
