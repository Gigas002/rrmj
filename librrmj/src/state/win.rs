use crate::Error;
use crate::action::Action;
use crate::event::Event;
use crate::rules::{RulesRegistry, WinContext, WinTimingFlags};
use crate::scoring::WinType;
use crate::state::{HandEndReason, HandPhase, HandState, next_seat};
use crate::tile::Tile;

impl HandState {
    pub fn config(&self) -> &crate::rules::RulesConfig {
        &self.config
    }

    pub fn scores(&self) -> &[i32; 4] {
        &self.scores
    }

    pub fn honba(&self) -> u8 {
        self.honba
    }

    pub fn table_riichi_sticks(&self) -> u8 {
        self.table_riichi_sticks
    }

    pub fn is_riichi(&self, seat: usize) -> bool {
        self.riichi[seat]
    }

    pub fn is_double_riichi(&self, seat: usize) -> bool {
        self.double_riichi[seat]
    }

    pub fn ippatsu_live(&self, seat: usize) -> bool {
        self.ippatsu_live[seat]
    }

    pub(crate) fn void_ippatsu(&mut self) {
        self.ippatsu_live = [false; 4];
    }

    pub(crate) fn is_haitei_win(&self, win_type: WinType) -> bool {
        matches!(win_type, WinType::Tsumo)
            && self.wall().live_remaining() == 0
            && !self.is_rinshan_draw
    }

    pub(crate) fn is_houtei_win(&self, win_type: WinType) -> bool {
        matches!(win_type, WinType::Ron { .. }) && self.wall().live_remaining() == 0
    }

    pub(crate) fn is_rinshan_win(&self, win_type: WinType) -> bool {
        matches!(win_type, WinType::Tsumo) && self.is_rinshan_draw
    }

    pub(crate) fn is_tenhou_win(&self, winner: usize, win_type: WinType) -> bool {
        winner == self.dealer()
            && self.is_dealer_first_turn
            && matches!(win_type, WinType::Tsumo)
            && (0..super::SEAT_COUNT).all(|seat| self.discards(seat).is_empty())
    }

    pub(crate) fn is_chiihou_win(&self, winner: usize, win_type: WinType) -> bool {
        winner != self.dealer()
            && matches!(win_type, WinType::Tsumo)
            && self.live_draws[winner] == 1
    }

    pub(crate) fn is_renhou_win(&self, winner: usize, win_type: WinType) -> bool {
        let WinType::Ron { from } = win_type else {
            return false;
        };
        winner == next_seat(self.dealer())
            && from == self.dealer()
            && self.live_draws[winner] == 0
            && !self.calls_made
            && self.first_discards[self.dealer()].is_some()
    }

    pub(crate) fn can_ron(&self, seat: usize) -> bool {
        let Some(reaction) = &self.reaction else {
            return false;
        };
        let win_tile = reaction.tile;
        if self.is_furiten(seat, win_tile) {
            return false;
        }
        let Ok(profile) = RulesRegistry::get(self.config.profile) else {
            return false;
        };
        let ctx = WinContext::new(
            self,
            seat,
            WinType::Ron {
                from: reaction.discarder,
            },
            win_tile,
            WinTimingFlags {
                is_chankan: reaction.kind == crate::state::reaction::ReactionKind::Kakan,
            },
        );
        profile.can_win(&ctx, &self.config)
    }

    pub(crate) fn can_tsumo(&self, seat: usize) -> bool {
        let Some(win_tile) = self.last_draw else {
            return false;
        };
        if seat != self.current_actor() {
            return false;
        }
        let Ok(profile) = RulesRegistry::get(self.config.profile) else {
            return false;
        };
        let ctx = WinContext::new(
            self,
            seat,
            WinType::Tsumo,
            win_tile,
            WinTimingFlags::default(),
        );
        profile.can_win(&ctx, &self.config)
    }

    pub(crate) fn can_declare_riichi(&self, seat: usize) -> bool {
        seat == self.current_actor()
            && self.phase() == HandPhase::Discard
            && !self.riichi[seat]
            && self.hand(seat).melds().is_empty()
            && RulesRegistry::get(self.config.profile)
                .is_ok_and(|p| p.is_tenpai(self.hand(seat), &self.config))
            && self.scores[seat] >= 1_000
    }

    pub(crate) fn resolve_win(
        &mut self,
        winner: usize,
        win_type: WinType,
        win_tile: Tile,
    ) -> Result<Vec<Event>, Error> {
        self.resolve_multi_ron(vec![winner], win_type, win_tile, WinTimingFlags::default())
    }

    pub(crate) fn resolve_multi_ron(
        &mut self,
        winners: Vec<usize>,
        win_type: WinType,
        win_tile: Tile,
        timing: WinTimingFlags,
    ) -> Result<Vec<Event>, Error> {
        if winners.is_empty() {
            return Err(Error::CannotWin);
        }

        let profile = RulesRegistry::get(self.config.profile)?;
        let mut events = Vec::new();
        let mut combined_deltas = [0i32; 4];

        for &winner in &winners {
            let ctx = WinContext::new(self, winner, win_type, win_tile, timing);
            if !profile.can_win(&ctx, &self.config) {
                return Err(Error::CannotWin);
            }
            let result = profile.score_win(&ctx, &self.config);
            for (score, delta) in combined_deltas.iter_mut().zip(result.deltas) {
                *score += delta;
            }
            events.push(Event::Won {
                seat: winner,
                han: result.han,
                fu: result.fu,
            });
        }

        self.apply_deltas(&combined_deltas);
        self.table_riichi_sticks = 0;
        self.phase = HandPhase::Ended;
        self.reaction = None;
        self.end_reason = Some(HandEndReason::Win {
            winners: winners.clone(),
        });
        events.push(Event::ScoresAdjusted {
            deltas: combined_deltas,
        });
        Ok(events)
    }

    pub(crate) fn resolve_exhaustive_draw(
        &mut self,
        mut prior: Vec<Event>,
    ) -> Result<Vec<Event>, Error> {
        let profile = RulesRegistry::get(self.config.profile)?;
        let deltas = profile.score_exhaustive_draw(self, &self.config);
        self.apply_deltas(&deltas);
        self.phase = HandPhase::Ended;
        self.reaction = None;
        self.end_reason = Some(HandEndReason::ExhaustiveDraw);
        prior.push(Event::ExhaustiveDraw { deltas });
        Ok(prior)
    }

    pub(crate) fn apply_riichi(&mut self, seat: usize, discard: Tile) -> Result<Vec<Event>, Error> {
        if !self.can_declare_riichi(seat) {
            return Err(Error::CannotDeclareRiichi);
        }
        let profile = RulesRegistry::get(self.config.profile)?;
        if !profile.is_riichi_discard(self.hand(seat), discard, &self.config) {
            return Err(Error::IllegalAction {
                action: Action::Riichi { discard },
                phase: self.phase(),
            });
        }
        let is_first_discard = self.first_discards[seat].is_none();
        self.scores[seat] -= 1_000;
        self.table_riichi_sticks += 1;
        self.riichi[seat] = true;
        self.ippatsu_live[seat] = true;
        if is_first_discard && !self.calls_made {
            self.double_riichi[seat] = true;
        }
        self.apply_discard_inner(seat, discard, true)
    }

    pub(crate) fn is_furiten(&self, seat: usize, win_tile: Tile) -> bool {
        if self.temporary_furiten[seat] || self.riichi_furiten[seat] {
            return true;
        }
        self.discards(seat)
            .iter()
            .any(|tile| tile.matches_identity(win_tile))
    }

    pub(crate) fn mark_passed_win(&mut self, seat: usize) {
        if !self.can_ron(seat) {
            return;
        }
        if self.riichi[seat] {
            self.riichi_furiten[seat] = true;
        } else {
            self.temporary_furiten[seat] = true;
        }
    }

    pub(crate) fn clear_temporary_furiten(&mut self, seat: usize) {
        self.temporary_furiten[seat] = false;
    }

    pub(crate) fn apply_deltas(&mut self, deltas: &[i32; 4]) {
        for (score, delta) in self.scores.iter_mut().zip(deltas) {
            *score += delta;
        }
    }
}
