use crate::Error;
use crate::event::Event;
use crate::game::{AbortiveDrawKind, AbortiveTrigger};
use crate::rules::RulesRegistry;
use crate::state::{HandEndReason, HandPhase, HandState};
use crate::tile::Tile;

impl HandState {
    pub(crate) fn can_abort_nine_terminals(&self, seat: usize) -> bool {
        if seat != self.dealer() || !self.is_dealer_first_turn {
            return false;
        }
        RulesRegistry::get(self.config.profile).is_ok_and(|profile| {
            profile.detect_abortive(self, &self.config, AbortiveTrigger::DealerFirstTurn)
                == Some(AbortiveDrawKind::NineTerminals)
        })
    }

    pub(crate) fn check_abortive_after_discard(
        &self,
        seat: usize,
        tile: Tile,
    ) -> Option<AbortiveDrawKind> {
        let profile = RulesRegistry::get(self.config.profile).ok()?;
        profile.detect_abortive(
            self,
            &self.config,
            AbortiveTrigger::FirstDiscard { seat, tile },
        )
    }

    pub(crate) fn check_abortive_after_kan(&self) -> Option<AbortiveDrawKind> {
        let profile = RulesRegistry::get(self.config.profile).ok()?;
        profile.detect_abortive(self, &self.config, AbortiveTrigger::KanDeclared)
    }

    pub(crate) fn check_abortive_after_riichi(&self) -> Option<AbortiveDrawKind> {
        let profile = RulesRegistry::get(self.config.profile).ok()?;
        profile.detect_abortive(self, &self.config, AbortiveTrigger::RiichiDeclared)
    }

    pub(crate) fn apply_abortive_nine_terminals(
        &mut self,
        seat: usize,
    ) -> Result<Vec<Event>, Error> {
        if seat != self.dealer() || !self.is_dealer_first_turn {
            return Err(Error::IllegalAction {
                action: crate::action::Action::AbortiveNineTerminals,
                phase: self.phase,
            });
        }
        if !self.can_abort_nine_terminals(seat) {
            return Err(Error::IllegalAction {
                action: crate::action::Action::AbortiveNineTerminals,
                phase: self.phase,
            });
        }
        self.resolve_abortive_draw(AbortiveDrawKind::NineTerminals, Vec::new())
    }

    pub(crate) fn resolve_abortive_draw(
        &mut self,
        kind: AbortiveDrawKind,
        mut prior: Vec<Event>,
    ) -> Result<Vec<Event>, Error> {
        self.phase = HandPhase::Ended;
        self.reaction = None;
        self.end_reason = Some(HandEndReason::AbortiveDraw(kind));
        prior.push(Event::AbortiveDraw { kind });
        Ok(prior)
    }
}
