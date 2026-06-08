use rand::rngs::StdRng;
use rand::SeedableRng;

use super::{HandPhase, HandState};
use crate::action::Action;
use crate::event::Event;
use crate::rules::RulesConfig;
use crate::wall::Wall;
use crate::Error;

#[test]
fn dealer_starts_in_discard_phase_with_fourteen_tiles() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(1));
    let deal = wall.deal(0).unwrap();
    let state = HandState::from_deal(wall, deal);

    assert_eq!(state.current_actor(), 0);
    assert_eq!(state.phase(), HandPhase::Discard);
    assert_eq!(state.hand(0).total_tiles(), 14);
    assert_eq!(state.hand(1).total_tiles(), 13);
}

#[test]
fn turn_rotates_draw_then_discard() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal);

    let tile = state.hand(0).concealed().tiles()[0];
    let events = state.apply(0, Action::Discard(tile)).unwrap();
    assert_eq!(
        events,
        vec![Event::Discarded {
            seat: 0,
            tile
        }]
    );
    assert_eq!(state.phase(), HandPhase::Reaction);

    for seat in 1..4 {
        state.apply(seat, Action::Pass).unwrap();
    }

    assert_eq!(state.current_actor(), 1);
    assert_eq!(state.phase(), HandPhase::Draw);

    let events = state.apply(1, Action::Draw).unwrap();
    assert!(matches!(events.as_slice(), [Event::Drawn { seat: 1, .. }]));
    assert_eq!(state.phase(), HandPhase::Discard);
    assert_eq!(state.hand(1).total_tiles(), 14);
}

#[test]
fn wrong_actor_and_phase_are_rejected() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(3));
    let deal = wall.deal(1).unwrap();
    let mut state = HandState::from_deal(wall, deal);

    assert!(matches!(
        state.apply(2, Action::Draw),
        Err(Error::WrongActor { .. })
    ));

    let tile = state.hand(1).concealed().tiles()[0];
    state.apply(1, Action::Discard(tile)).unwrap();

    assert!(matches!(
        state.apply(2, Action::Draw),
        Err(Error::IllegalAction { .. })
    ));
}

#[test]
fn scripted_play_conserves_tiles() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(99));
    let deal = wall.deal(2).unwrap();
    let mut state = HandState::from_deal(wall, deal);

    let events = state
        .play_out_discards(|hand_state, seat| hand_state.hand(seat).concealed().tiles()[0])
        .unwrap();

    assert!(state.is_ended());
    assert!(events.iter().any(|e| matches!(e, Event::HandEnded)));
    assert!(events.iter().filter(|e| matches!(e, Event::Drawn { .. })).count() > 10);
    state.validate_tile_conservation().unwrap();
}

#[test]
fn pass_is_illegal_during_dealer_discard() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(4));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal);

    assert!(matches!(
        state.apply(0, Action::Pass),
        Err(Error::IllegalAction { .. })
    ));
}
