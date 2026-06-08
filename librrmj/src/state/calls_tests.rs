use rand::rngs::StdRng;
use rand::SeedableRng;

use super::{HandPhase, HandState};
use crate::action::Action;
use crate::event::Event;
use crate::hand::MeldKind;
use crate::rules::RulesConfig;
use crate::tile::{Suit, Tile};
use crate::wall::Wall;
use crate::Error;

fn build_hand(seat: usize, mut tiles: Vec<Tile>) -> Vec<Tile> {
    let mut n = 0usize;
    while tiles.len() < 13 {
        let rank = ((n + seat) % 9) + 1;
        let suit = match ((n + seat) / 9) % 3 {
            0 => Suit::Man,
            1 => Suit::Pin,
            _ => Suit::Sou,
        };
        let candidate = Tile::numbered(suit, rank as u8);
        if !tiles.contains(&candidate) {
            tiles.push(candidate);
        }
        n += 1;
    }
    tiles
}

fn start_reaction(
    seed: u64,
    dealer: usize,
    discarded: Tile,
    hands: [Vec<Tile>; 4],
) -> HandState {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(seed));
    let deal = wall.deal(dealer).unwrap();
    let mut state = HandState::from_deal(wall, deal);

    let mut configured = hands;
    if !configured[dealer].contains(&discarded) {
        configured[dealer].push(discarded);
    }
    for (seat, tiles) in configured.into_iter().enumerate() {
        state.set_concealed(seat, build_hand(seat, tiles));
    }
    state.apply(dealer, Action::Discard(discarded)).unwrap();
    state
}

fn pass_all_except(state: &mut HandState, skip: usize) {
    for seat in 0..4 {
        if seat != skip
            && state.phase() == HandPhase::Reaction
            && state.legal_actions_for(seat).contains(&Action::Pass)
        {
            state.apply(seat, Action::Pass).unwrap();
        }
    }
}

#[test]
fn pon_call_adds_open_meld_and_skips_draw() {
    let discarded = Tile::man(2);
    let mut state = start_reaction(
        10,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(2), Tile::man(2), Tile::pin(9)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );

    state.apply(1, Action::Pon).unwrap();
    pass_all_except(&mut state, 1);

    assert_eq!(state.phase(), HandPhase::Discard);
    assert_eq!(state.current_actor(), 1);
    assert_eq!(state.hand(1).melds().len(), 1);
    assert_eq!(state.hand(1).melds()[0].kind(), MeldKind::Pon);
    assert_eq!(state.hand(1).total_tiles(), 14);
    assert!(state.discards(0).is_empty());
}

#[test]
fn chi_only_from_kamicha() {
    let discarded = Tile::man(2);
    let mut state = start_reaction(
        11,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );

    let chi = Action::Chi {
        tiles: [Tile::man(1), discarded, Tile::man(3)],
    };
    state.apply(1, chi).unwrap();
    pass_all_except(&mut state, 1);

    assert_eq!(state.current_actor(), 1);
    assert_eq!(state.hand(1).melds()[0].kind(), MeldKind::Chi);

    let mut state = start_reaction(
        12,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    let chi = Action::Chi {
        tiles: [Tile::man(1), discarded, Tile::man(3)],
    };
    assert!(matches!(
        state.apply(2, chi),
        Err(Error::InvalidCall { .. })
    ));
}

#[test]
fn open_kan_reveals_dora_and_draws_rinshan() {
    let discarded = Tile::sou(6);
    let mut state = start_reaction(
        13,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![
                Tile::sou(6),
                Tile::sou(6),
                Tile::sou(6),
                Tile::pin(9),
            ],
            vec![Tile::man(1), Tile::man(2), Tile::man(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(7)],
        ],
    );

    state.apply(1, Action::OpenKan).unwrap();
    state.apply(2, Action::Pass).unwrap();
    let events = state.apply(3, Action::Pass).unwrap();

    assert!(events
        .iter()
        .any(|event| matches!(event, Event::DoraRevealed { .. })));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::RinshanDrawn { seat: 1, .. })));
    assert_eq!(state.hand(1).melds()[0].kind(), MeldKind::OpenKan);
    assert_eq!(state.hand(1).total_tiles(), 15);
    assert_eq!(state.wall().kan_count(), 1);
}

#[test]
fn pon_beats_chi_when_both_claim() {
    let discarded = Tile::man(2);
    let mut state = start_reaction(
        14,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::man(2), Tile::man(2), Tile::pin(8)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );

    let chi = Action::Chi {
        tiles: [Tile::man(1), discarded, Tile::man(3)],
    };
    state.apply(1, chi).unwrap();
    state.apply(2, Action::Pon).unwrap();
    state.apply(3, Action::Pass).unwrap();

    assert_eq!(state.current_actor(), 2);
    assert_eq!(state.hand(2).melds()[0].kind(), MeldKind::Pon);
    assert!(state.hand(1).melds().is_empty());
}

#[test]
fn illegal_pon_without_tiles_is_rejected() {
    let discarded = Tile::man(2);
    let mut state = start_reaction(
        15,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(4), Tile::pin(9), Tile::pin(1)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );

    assert!(matches!(
        state.apply(1, Action::Pon),
        Err(Error::InvalidCall { .. })
    ));
}

#[test]
fn closed_kan_on_own_turn_stays_on_discard() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(16));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal);

    state.set_concealed(
        0,
        vec![
            Tile::pin(3),
            Tile::pin(3),
            Tile::pin(3),
            Tile::pin(3),
            Tile::man(1),
            Tile::man(2),
            Tile::man(4),
            Tile::man(5),
            Tile::man(6),
            Tile::man(7),
            Tile::man(8),
            Tile::man(9),
            Tile::sou(1),
            Tile::sou(2),
        ],
    );

    let events = state
        .apply(0, Action::ClosedKan { tile: Tile::pin(3) })
        .unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::DoraRevealed { .. })));
    assert_eq!(state.phase(), HandPhase::Discard);
    assert_eq!(state.current_actor(), 0);
    assert_eq!(state.hand(0).melds()[0].kind(), MeldKind::ClosedKan);
}
