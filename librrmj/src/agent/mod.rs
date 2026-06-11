mod slot;
mod view;

#[cfg(test)]
mod tests;

use crate::action::Action;

pub use slot::PlayerSlot;
pub use view::{PendingCall, PlayerView, SeatView, TurnContext, TurnFocus};

/// Chooses a legal action given what a seat is allowed to see.
pub trait Agent {
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action;
}

/// Adapter for closures used in tests and simple clients.
pub struct FnAgent<F>(pub F);

impl<F> Agent for FnAgent<F>
where
    F: FnMut(&PlayerView, &[Action]) -> Action,
{
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action {
        (self.0)(view, legal)
    }
}
