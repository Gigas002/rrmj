use rand::SeedableRng;
use rand::rngs::StdRng;

use super::{HandPhase, HandState};
use crate::Error;
use crate::action::{Action, KanIntent};
use crate::event::Event;
use crate::hand::{Hand, Meld, MeldKind};
use crate::rules::{RulesConfig, RulesProfileId, RulesRegistry};
use crate::test_util::fixtures::{
    tenpai_after_draw_p2, tenpai_waiting_on_p2, winning_tanyao_tiles,
};
use crate::tile::{Suit, Tile};
use crate::wall::Wall;

// --- turn flow ---

#[test]
fn dealer_starts_in_discard_phase_with_fourteen_tiles() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(1));
    let deal = wall.deal(0).unwrap();
    let state = HandState::from_deal(wall, deal, rules);

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
    let mut state = HandState::from_deal(wall, deal, rules);

    let tile = state.hand(0).concealed().tiles()[0];
    let events = state.apply(0, Action::Discard(tile)).unwrap();
    assert_eq!(events, vec![Event::Discarded { seat: 0, tile }]);
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
    let mut state = HandState::from_deal(wall, deal, rules);

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
    let mut state = HandState::from_deal(wall, deal, rules);

    let events = state
        .play_out_discards(|hand_state, seat| hand_state.hand(seat).concealed().tiles()[0])
        .unwrap();

    assert!(state.is_ended());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ExhaustiveDraw { .. }))
    );
    assert!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Drawn { .. }))
            .count()
            > 10
    );
    state.validate_tile_conservation().unwrap();
}

#[test]
fn pass_is_illegal_during_dealer_discard() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(4));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, rules);

    assert!(matches!(
        state.apply(0, Action::Pass),
        Err(Error::IllegalAction { .. })
    ));
}

// --- calls ---

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

fn start_reaction(seed: u64, dealer: usize, discarded: Tile, hands: [Vec<Tile>; 4]) -> HandState {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(seed));
    let deal = wall.deal(dealer).unwrap();
    let mut state = HandState::from_deal(wall, deal, rules);

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
            vec![Tile::sou(6), Tile::sou(6), Tile::sou(6), Tile::pin(9)],
            vec![Tile::man(1), Tile::man(2), Tile::man(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(7)],
        ],
    );

    state.apply(1, Action::Kan(KanIntent::Open)).unwrap();
    state.apply(2, Action::Pass).unwrap();
    let events = state.apply(3, Action::Pass).unwrap();

    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::DoraRevealed { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::RinshanDrawn { seat: 1, .. }))
    );
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
    let mut state = HandState::from_deal(wall, deal, rules);

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
        .apply(0, Action::Kan(KanIntent::Closed { tile: Tile::pin(3) }))
        .unwrap();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::DoraRevealed { .. }))
    );
    assert_eq!(state.phase(), HandPhase::Discard);
    assert_eq!(state.current_actor(), 0);
    assert_eq!(state.hand(0).melds()[0].kind(), MeldKind::ClosedKan);
}

// --- wins ---

#[test]
fn tsumo_ends_hand_and_adjusts_scores() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(42));
    let deal = wall.deal(1).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_concealed(1, winning_tanyao_tiles());
    state.last_draw = Some(Tile::pin(2));

    let events = state.apply(1, Action::Tsumo).unwrap();
    assert!(state.is_ended());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Won { seat: 1, .. }))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ScoresAdjusted { .. }))
    );
    assert!(state.scores()[1] > state.config().starting_points);
}

#[test]
fn ron_on_discard_wins_immediately() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(7));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(1, tenpai_waiting_on_p2());
    let mut dealer_hand: Vec<Tile> = state.hand(0).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    state.set_concealed(0, dealer_hand);

    let discard = Tile::pin(2);
    state.apply(0, Action::Discard(discard)).unwrap();
    state.apply(1, Action::Ron).unwrap();
    state.apply(2, Action::Pass).unwrap();
    let events = state.apply(3, Action::Pass).unwrap();

    assert!(state.is_ended());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Won { seat: 1, .. }))
    );
    assert!(state.scores()[0] < state.config().starting_points);
    assert!(state.scores()[1] > state.config().starting_points);
}

#[test]
fn riichi_legal_actions_only_include_tenpai_preserving_discards() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(11));
    let deal = wall.deal(2).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(2, tenpai_after_draw_p2());
    state.last_draw = Some(Tile::pin(2));

    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();
    let config = state.config();
    assert!(profile.is_tenpai(state.hand(2), config));
    assert!(profile.is_riichi_discard(state.hand(2), Tile::pin(2), config));
    assert!(!profile.is_riichi_discard(state.hand(2), Tile::man(9), config));

    let riichi_discards: Vec<Tile> = state
        .legal_actions_for(2)
        .into_iter()
        .filter_map(|a| match a {
            Action::Riichi { discard } => Some(discard),
            _ => None,
        })
        .collect();

    assert!(riichi_discards.contains(&Tile::pin(2)));
    assert!(!riichi_discards.is_empty());
    for discard in &riichi_discards {
        assert!(profile.is_riichi_discard(state.hand(2), *discard, config));
    }
}

#[test]
fn riichi_rejects_discard_that_breaks_tenpai() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(11));
    let deal = wall.deal(2).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(2, tenpai_after_draw_p2());
    state.last_draw = Some(Tile::pin(2));

    let err = state
        .apply(
            2,
            Action::Riichi {
                discard: Tile::man(9),
            },
        )
        .unwrap_err();
    assert!(matches!(err, Error::IllegalAction { .. }));
}

#[test]
fn riichi_declaration_costs_stick_and_ends_in_reaction() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(11));
    let deal = wall.deal(2).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(2, tenpai_after_draw_p2());
    state.last_draw = Some(Tile::pin(2));

    let discard = Tile::pin(2);
    let events = state.apply(2, Action::Riichi { discard }).unwrap();

    assert!(state.is_riichi(2));
    assert_eq!(state.table_riichi_sticks(), 1);
    assert_eq!(state.scores()[2], state.config().starting_points - 1_000);
    assert_eq!(state.phase(), HandPhase::Reaction);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::RiichiDeclared { .. }))
    );
    assert!(state.ippatsu_live(2));
}

#[test]
fn double_riichi_on_first_discard_before_any_call() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(21));
    let deal = wall.deal(2).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(2, tenpai_after_draw_p2());
    state.last_draw = Some(Tile::pin(2));

    state
        .apply(
            2,
            Action::Riichi {
                discard: Tile::pin(2),
            },
        )
        .unwrap();

    assert!(state.is_double_riichi(2));
}

#[test]
fn call_after_riichi_voids_ippatsu() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(22));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(0, tenpai_after_draw_p2());
    state.last_draw = Some(Tile::pin(2));
    state
        .apply(
            0,
            Action::Riichi {
                discard: Tile::pin(2),
            },
        )
        .unwrap();
    assert!(state.ippatsu_live(0));

    state.set_concealed(
        1,
        vec![
            Tile::pin(1),
            Tile::pin(3),
            Tile::man(1),
            Tile::man(2),
            Tile::man(3),
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
    state
        .apply(
            1,
            Action::Chi {
                tiles: [Tile::pin(1), Tile::pin(2), Tile::pin(3)],
            },
        )
        .unwrap();
    for seat in 0..4 {
        let _ = state.apply(seat, Action::Pass);
    }

    assert!(!state.ippatsu_live(0));
}

#[test]
fn furiten_blocks_ron() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(13));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(1, tenpai_waiting_on_p2());
    state.push_discard_for_test(1, Tile::pin(2));

    let mut dealer_hand: Vec<Tile> = state.hand(0).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    state.set_concealed(0, dealer_hand);

    state.apply(0, Action::Discard(Tile::pin(2))).unwrap();
    assert!(matches!(
        state.apply(1, Action::Ron),
        Err(crate::Error::Furiten)
    ));
}

#[test]
fn kakan_upgrades_pon_and_reveals_dora() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(20));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    let pon = Meld::pon([Tile::sou(5), Tile::sou(5), Tile::sou(5)], Tile::sou(5)).unwrap();
    let hand = Hand::new(
        crate::hand::Concealed::from_tiles(vec![
            Tile::sou(5),
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::man(6),
            Tile::man(7),
            Tile::man(8),
            Tile::pin(2),
            Tile::pin(3),
            Tile::pin(4),
            Tile::pin(6),
            Tile::pin(7),
            Tile::pin(8),
        ]),
        vec![pon],
    )
    .unwrap();
    state.set_hand(0, hand);

    let events = state
        .apply(0, Action::Kan(KanIntent::Added { meld_index: 0 }))
        .unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::KakanDeclared { .. }))
    );

    state.apply(1, Action::Pass).unwrap();
    state.apply(2, Action::Pass).unwrap();
    let events = state.apply(3, Action::Pass).unwrap();

    assert_eq!(state.hand(0).melds()[0].kind(), MeldKind::OpenKan);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::DoraRevealed { .. }))
    );
    assert_eq!(state.phase(), HandPhase::Discard);
    assert_eq!(state.current_actor(), 0);
}

#[test]
fn chankan_ron_on_kakan_tile() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(21));
    let deal = wall.deal(1).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    let pon = Meld::pon([Tile::sou(5), Tile::sou(5), Tile::sou(5)], Tile::sou(5)).unwrap();
    state.set_hand(
        1,
        Hand::new(
            crate::hand::Concealed::from_tiles(vec![
                Tile::sou(5),
                Tile::man(2),
                Tile::man(3),
                Tile::man(4),
                Tile::man(6),
                Tile::man(7),
                Tile::man(8),
                Tile::pin(2),
                Tile::pin(3),
                Tile::pin(4),
                Tile::pin(6),
                Tile::pin(7),
                Tile::pin(8),
            ]),
            vec![pon],
        )
        .unwrap(),
    );
    state.set_concealed(
        2,
        vec![
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::pin(3),
            Tile::pin(4),
            Tile::pin(5),
            Tile::sou(6),
            Tile::sou(7),
            Tile::sou(8),
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(9),
            Tile::sou(9),
        ],
    );

    state
        .apply(1, Action::Kan(KanIntent::Added { meld_index: 0 }))
        .unwrap();
    state.apply(0, Action::Pass).unwrap();
    state.apply(2, Action::Ron).unwrap();
    let events = state.apply(3, Action::Pass).unwrap();

    assert!(state.is_ended());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Won { seat: 2, .. }))
    );
}

#[test]
fn passing_on_win_sets_temporary_furiten() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(22));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(1, tenpai_waiting_on_p2());
    let mut dealer_hand: Vec<Tile> = state.hand(0).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    state.set_concealed(0, dealer_hand);

    state.apply(0, Action::Discard(Tile::pin(2))).unwrap();
    assert!(state.can_ron(1));
    state.apply(1, Action::Pass).unwrap();
    assert!(!state.can_ron(1));
}

#[test]
fn draw_clears_temporary_furiten() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(221));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(1, tenpai_waiting_on_p2());
    let mut dealer_hand: Vec<Tile> = state.hand(0).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    state.set_concealed(0, dealer_hand);

    state.apply(0, Action::Discard(Tile::pin(2))).unwrap();
    state.apply(1, Action::Pass).unwrap();
    state.apply(2, Action::Pass).unwrap();
    state.apply(3, Action::Pass).unwrap();

    assert_eq!(state.current_actor(), 1);
    state.apply(1, Action::Draw).unwrap();
    assert!(!state.is_furiten(1, Tile::pin(2)));
}

#[test]
fn riichi_furiten_persists_after_pass() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(23));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.riichi[1] = true;
    state.set_concealed(1, tenpai_waiting_on_p2());
    let mut dealer_hand: Vec<Tile> = state.hand(0).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    state.set_concealed(0, dealer_hand);

    state.apply(0, Action::Discard(Tile::pin(2))).unwrap();
    assert!(state.can_ron(1));
    state.apply(1, Action::Pass).unwrap();
    assert!(!state.can_ron(1));

    state.apply(2, Action::Pass).unwrap();
    state.apply(3, Action::Pass).unwrap();
    state.apply(1, Action::Draw).unwrap();
    assert!(state.is_furiten(1, Tile::pin(2)));
}

#[test]
fn double_ron_when_enabled() {
    let mut config = RulesConfig::standard();
    config.double_ron = true;
    let starting = config.starting_points;
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(24));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    state.set_concealed(1, tenpai_waiting_on_p2());
    state.set_concealed(3, tenpai_waiting_on_p2());
    let mut dealer_hand: Vec<Tile> = state.hand(0).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    state.set_concealed(0, dealer_hand);

    state.apply(0, Action::Discard(Tile::pin(2))).unwrap();
    state.apply(1, Action::Ron).unwrap();
    state.apply(2, Action::Pass).unwrap();
    state.apply(3, Action::Ron).unwrap();

    assert!(state.is_ended());
    assert!(matches!(
        state.end_reason(),
        Some(crate::state::HandEndReason::Win { winners })
            if winners == vec![1usize, 3]
    ));
    assert!(state.scores()[0] < starting);
    assert!(state.scores()[1] > starting);
    assert!(state.scores()[3] > starting);
}
