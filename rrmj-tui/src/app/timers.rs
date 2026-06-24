use std::time::{Duration, Instant};

use librrmj::action::Action;
use librrmj::state::HandPhase;

use crate::app::ActionMenu;
use crate::timers::{SeatTimer, TimerKind};

/// Tracks the action deadline for whoever must act (`pending_seat`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TimerAnchor {
    seat: usize,
    phase: HandPhase,
}

#[derive(Debug, Default)]
pub struct ActionTimerState {
    anchor: Option<TimerAnchor>,
    deadline: Option<Instant>,
    limit_ms: u64,
}

impl ActionTimerState {
    pub fn reset(&mut self) {
        self.anchor = None;
        self.deadline = None;
        self.limit_ms = 0;
    }

    pub fn sync(
        &mut self,
        seat: usize,
        phase: HandPhase,
        turn_timer_ms: u64,
        response_timer_ms: u64,
    ) {
        let anchor = TimerAnchor { seat, phase };
        if self.anchor == Some(anchor) {
            return;
        }
        self.anchor = Some(anchor);
        let limit_ms = match phase {
            HandPhase::Draw => 0,
            HandPhase::Discard => turn_timer_ms,
            HandPhase::Reaction => response_timer_ms,
            HandPhase::Ended => 0,
        };
        self.limit_ms = limit_ms;
        self.deadline = if limit_ms == 0 {
            None
        } else {
            Some(Instant::now() + Duration::from_millis(limit_ms))
        };
    }

    pub fn remaining_ms(&self) -> Option<u64> {
        let deadline = self.deadline?;
        let remaining = deadline.saturating_duration_since(Instant::now());
        Some(remaining.as_millis().min(self.limit_ms as u128) as u64)
    }

    pub fn is_expired(&self) -> bool {
        self.deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
    }

    pub fn seat_timer(&self, kind: TimerKind) -> Option<SeatTimer> {
        let remaining_ms = self.remaining_ms()?;
        Some(SeatTimer {
            kind,
            remaining_ms,
            total_ms: self.limit_ms,
        })
    }
}

pub fn timeout_action(legal: &[Action], phase: HandPhase, menu: &ActionMenu) -> Option<Action> {
    match phase {
        HandPhase::Reaction => Some(Action::Pass),
        HandPhase::Draw => Some(Action::Draw),
        HandPhase::Discard => {
            if menu.can_tsumo {
                return Some(Action::Tsumo);
            }
            menu.discards.first().copied().map(Action::Discard)
        }
        HandPhase::Ended => None,
    }
    .filter(|action| legal.contains(action))
}
