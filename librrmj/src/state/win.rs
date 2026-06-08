use crate::Error;
use crate::event::Event;
use crate::rules::{RulesRegistry, WinContext};
use crate::scoring::WinType;
use crate::state::{HandEndReason, HandPhase, HandState};
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
        let ctx = WinContext {
            state: self,
            winner: seat,
            win_type: WinType::Ron {
                from: reaction.discarder,
            },
            win_tile,
        };
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
        let ctx = WinContext {
            state: self,
            winner: seat,
            win_type: WinType::Tsumo,
            win_tile,
        };
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
        let profile = RulesRegistry::get(self.config.profile)?;
        let ctx = WinContext {
            state: self,
            winner,
            win_type,
            win_tile,
        };
        if !profile.can_win(&ctx, &self.config) {
            return Err(Error::CannotWin);
        }

        let result = profile.score_win(&ctx, &self.config);
        self.apply_deltas(&result.deltas);
        self.table_riichi_sticks = 0;
        self.phase = HandPhase::Ended;
        self.reaction = None;
        self.end_reason = Some(HandEndReason::Win { winner });

        Ok(vec![
            Event::Won {
                seat: winner,
                han: result.han,
                fu: result.fu,
            },
            Event::ScoresAdjusted {
                deltas: result.deltas,
            },
        ])
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
        self.scores[seat] -= 1_000;
        self.table_riichi_sticks += 1;
        self.riichi[seat] = true;
        self.apply_discard_inner(seat, discard, true)
    }

    pub(crate) fn is_furiten(&self, seat: usize, win_tile: Tile) -> bool {
        self.discards(seat)
            .iter()
            .any(|tile| tile.matches_identity(win_tile))
    }

    pub(crate) fn apply_deltas(&mut self, deltas: &[i32; 4]) {
        for (score, delta) in self.scores.iter_mut().zip(deltas) {
            *score += delta;
        }
    }
}
