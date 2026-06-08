mod calls;
mod hand_state;
mod reaction;
mod win;

#[cfg(test)]
mod tests;

pub use calls::CallKind;
pub use hand_state::{HandPhase, HandState, SEAT_COUNT};

pub fn next_seat(seat: usize) -> usize {
    debug_assert!(seat < SEAT_COUNT);
    (seat + 1) % SEAT_COUNT
}
