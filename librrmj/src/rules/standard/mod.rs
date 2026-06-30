mod abortive;
pub mod dora;
mod fu;
mod patterns;
mod recommendations;
mod score;
mod win;
mod yaku;

#[cfg(test)]
mod tests;

use crate::game::{AbortiveDrawKind, AbortiveTrigger};
use crate::hand::Hand;
use crate::rules::RulesConfig;
use crate::rules::RulesProfileId;
use crate::rules::flow::{GameFlowPolicy, StandardGameFlow};
use crate::rules::profile::{RulesProfile, WinContext};
use crate::rules::recommendations::{Recommendation, sort_recommendations};
use crate::rules::standard::recommendations::collect;
use crate::scoring::ScoringResult;
use crate::state::HandState;
use crate::tile::Tile;

#[cfg(feature = "ai")]
pub use win::is_winning_hand;

pub struct StandardRules;

impl RulesProfile for StandardRules {
    fn id(&self) -> RulesProfileId {
        RulesProfileId::Standard
    }

    fn can_win(&self, ctx: &WinContext<'_>, _config: &RulesConfig) -> bool {
        win::is_winning_hand(ctx.hand(), Some(ctx.win_tile))
    }

    fn is_tenpai(&self, hand: &Hand, _config: &RulesConfig) -> bool {
        win::is_tenpai(hand)
    }

    fn is_riichi_discard(&self, hand: &Hand, discard: Tile, _config: &RulesConfig) -> bool {
        win::is_tenpai_after_discard(hand, discard)
    }

    fn score_win(&self, ctx: &WinContext<'_>, config: &RulesConfig) -> ScoringResult {
        let yaku = yaku::detect_yaku(ctx, config);
        let dora = dora::count_dora(ctx, config);
        let ura_dora = dora::count_ura_dora(ctx, config);
        let aka_dora = dora::count_aka_dora(ctx, config);
        let is_open = !ctx.hand().melds().is_empty();
        let han = yaku::total_han(&yaku, is_open) + dora + ura_dora + aka_dora;
        let fu = fu::calculate_fu(ctx, &yaku, config);
        score::score_hand(ctx, &yaku, han, fu, dora, ura_dora, aka_dora, config)
    }

    fn recommendations(
        &self,
        state: &HandState,
        seat: usize,
        config: &RulesConfig,
        limit: usize,
    ) -> Vec<Recommendation> {
        let mut paths = collect(state, seat, config);
        sort_recommendations(&mut paths);
        paths.truncate(limit);
        paths
    }

    fn score_exhaustive_draw(&self, state: &HandState, _config: &RulesConfig) -> [i32; 4] {
        score::score_exhaustive_draw(state)
    }

    fn detect_abortive(
        &self,
        state: &HandState,
        config: &RulesConfig,
        trigger: AbortiveTrigger,
    ) -> Option<AbortiveDrawKind> {
        abortive::detect_abortive(state, config, trigger)
    }

    fn game_flow(&self) -> &dyn GameFlowPolicy {
        &StandardGameFlow
    }
}
