use crate::action::Action;
use crate::state::calls::{call_priority, seat_priority, CallKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionState {
    pub discarder: usize,
    pub tile: crate::tile::Tile,
    responses: [Option<Action>; 4],
}

impl ReactionState {
    pub fn new(discarder: usize, tile: crate::tile::Tile) -> Self {
        Self {
            discarder,
            tile,
            responses: [None, None, None, None],
        }
    }

    pub fn can_respond(&self, seat: usize) -> bool {
        seat != self.discarder && self.responses[seat].is_none()
    }

    pub fn record(&mut self, seat: usize, action: Action) {
        self.responses[seat] = Some(action);
    }

    pub fn is_complete(&self) -> bool {
        (0..4).all(|seat| seat == self.discarder || self.responses[seat].is_some())
    }

    pub fn winning_call(&self) -> Option<(usize, Action)> {
        if !self.is_complete() {
            return None;
        }

        let mut best: Option<(usize, Action, u8, u8)> = None;
        for seat in 0..4 {
            if seat == self.discarder {
                continue;
            }
            let Some(action) = self.responses[seat] else {
                continue;
            };
            let Some(kind) = call_kind(action) else {
                continue;
            };

            let priority = call_priority(kind);
            let distance = seat_priority(self.discarder, seat);
            if best.is_none_or(|(_, _, p, d)| priority > p || (priority == p && distance < d)) {
                best = Some((seat, action, priority, distance));
            }
        }

        best.map(|(seat, action, _, _)| (seat, action))
    }
}

pub fn call_kind(action: Action) -> Option<CallKind> {
    match action {
        Action::Chi { .. } => Some(CallKind::Chi),
        Action::Pon => Some(CallKind::Pon),
        Action::OpenKan => Some(CallKind::OpenKan),
        _ => None,
    }
}
