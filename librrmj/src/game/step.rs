use crate::Error;
use crate::action::Action;
use crate::agent::{Agent, PlayerView};
use crate::event::Event;

use super::Game;

/// Result of advancing the match by one agent decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepResult {
    pub seat: usize,
    pub action: Action,
    pub events: Vec<Event>,
}

impl Game {
    /// Seat that must act next, if any.
    pub fn pending_seat(&self) -> Option<usize> {
        if self.is_ended() {
            return None;
        }
        match self.hand.phase() {
            crate::state::HandPhase::Draw | crate::state::HandPhase::Discard => {
                Some(self.hand.current_actor())
            }
            crate::state::HandPhase::Reaction => self.hand.pending_reaction_seat(),
            crate::state::HandPhase::Ended => None,
        }
    }

    /// Legal actions for the seat that must act next.
    pub fn pending_legal_actions(&self) -> Vec<Action> {
        self.pending_seat()
            .map(|seat| self.hand.legal_actions_for(seat))
            .unwrap_or_default()
    }

    /// Ask the appropriate agent for a decision and apply it.
    pub fn step<A: Agent>(&mut self, agents: &mut [A; 4]) -> Result<Option<StepResult>, Error> {
        let seat = match self.pending_seat() {
            Some(seat) => seat,
            None => return Ok(None),
        };

        let legal = self.hand.legal_actions_for(seat);
        if legal.is_empty() {
            return Ok(None);
        }

        let view = PlayerView::from_game(self, seat);
        let action = agents[seat].decide(&view, &legal);
        let events = self.apply_action(seat, action)?;

        Ok(Some(StepResult {
            seat,
            action,
            events,
        }))
    }
}
