use rand::SeedableRng;
use rand::rngs::StdRng;

use super::{HandPhase, HandState};
use crate::Error;
use crate::action::{Action, KanIntent};
use crate::event::Event;
use crate::hand::{Hand, KanForm, Meld, MeldKind};
use crate::rules::{RulesConfig, RulesProfileId, RulesRegistry};
use crate::tile::{Suit, Tile};
use crate::wall::Wall;

fn winning_tanyao_tiles() -> Vec<Tile> {
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
        Tile::sou(3),
        Tile::sou(4),
        Tile::sou(5),
        Tile::pin(2),
        Tile::pin(2),
    ]
}

fn tenpai_waiting_on_p2() -> Vec<Tile> {
    let mut hand = winning_tanyao_tiles();
    hand.pop();
    hand
}

fn tenpai_after_draw_p2() -> Vec<Tile> {
    let mut hand = tenpai_waiting_on_p2();
    hand.push(Tile::pin(2));
    hand
}

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
    assert_eq!(
        state.hand(1).melds()[0].kind(),
        MeldKind::Kan(KanForm::Open)
    );
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
    assert_eq!(
        state.hand(0).melds()[0].kind(),
        MeldKind::Kan(KanForm::Closed)
    );
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

    assert_eq!(
        state.hand(0).melds()[0].kind(),
        MeldKind::Kan(KanForm::Open)
    );
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

// --- audit §2 regressions ---

fn tiles(spec: &str) -> Vec<Tile> {
    spec.split_whitespace()
        .map(|t| t.parse().unwrap())
        .collect()
}

/// Dealer (seat 0) discards `tile` on the first turn.
fn dealer_discards(state: &mut HandState, tile: Tile) {
    let mut dealer_hand: Vec<Tile> = state.hand(0).concealed().tiles().to_vec();
    dealer_hand[0] = tile;
    state.set_concealed(0, dealer_hand);
    state.apply(0, Action::Discard(tile)).unwrap();
}

#[test]
fn ron_needs_a_yaku() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(31));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);

    // Open hand, no yaku: 123m 12p 666p WW + chi 234s, waiting on 3p.
    let chi = Meld::chi([Tile::sou(2), Tile::sou(3), Tile::sou(4)], Tile::sou(3)).unwrap();
    let concealed = crate::hand::Concealed::from_tiles(tiles("1m 2m 3m 1p 2p 6p 6p 6p W W"));
    state.set_hand(1, Hand::new(concealed, vec![chi]).unwrap());
    state.calls_made = true;

    dealer_discards(&mut state, Tile::pin(3));
    assert!(!state.legal_actions_for(1).contains(&Action::Ron));
    assert!(matches!(state.apply(1, Action::Ron), Err(Error::CannotWin)));
}

#[test]
fn round_wind_counts_for_yakuhai() {
    for (round, can_ron) in [
        (crate::game::RoundWind::East, false),
        (crate::game::RoundWind::South, true),
    ] {
        let config = RulesConfig::standard();
        let mut wall = Wall::new(&config, StdRng::seed_from_u64(32));
        let deal = wall.deal(0).unwrap();
        let mut state = HandState::from_deal(wall, deal, config);
        state.set_round_wind(round);
        // Seat 2 is West: a South triplet is yakuhai only in the South round.
        state.set_concealed(2, tiles("S S S 2m 3m 4m 3p 4p 5p 6s 7s 8s 9p"));
        dealer_discards(&mut state, Tile::pin(9));
        assert_eq!(
            state.legal_actions_for(2).contains(&Action::Ron),
            can_ron,
            "{round:?}"
        );
    }
}

#[test]
fn riichi_is_offered_when_a_discard_leaves_tenpai() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(33));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    // Tenpai after discarding East (waits 1p/4p), but not complete.
    state.set_concealed(0, tiles("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 2p 3p 4p E"));
    let legal = state.legal_actions_for(0);
    assert!(legal.contains(&Action::Riichi {
        discard: Tile::wind(crate::tile::Wind::East)
    }));
    assert!(!legal.contains(&Action::Tsumo));
}

/// Plays other seats (passing, drawing, discarding their first tile) until `seat` must discard.
fn advance_to_discard(state: &mut HandState, seat: usize) {
    for _ in 0..64 {
        if state.phase() == HandPhase::Discard && state.current_actor() == seat {
            return;
        }
        match state.phase() {
            HandPhase::Reaction => {
                let actor = state.pending_reaction_seat().unwrap();
                state.apply(actor, Action::Pass).unwrap();
            }
            HandPhase::Draw => {
                let actor = state.current_actor();
                state.apply(actor, Action::Draw).unwrap();
            }
            HandPhase::Discard => {
                let actor = state.current_actor();
                let discard = state
                    .legal_actions_for(actor)
                    .into_iter()
                    .find(|a| matches!(a, Action::Discard(_)))
                    .unwrap();
                state.apply(actor, discard).unwrap();
            }
            HandPhase::Ended => panic!("hand ended"),
        }
    }
    panic!("seat {seat} never reached its discard");
}

#[test]
fn riichi_restricts_discards_and_calls() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(34));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_concealed(0, tiles("1m 2m 3m 4m 5m 6m 7m 8m 9m 1p 2p 3p 4p E"));
    state
        .apply(
            0,
            Action::Riichi {
                discard: Tile::wind(crate::tile::Wind::East),
            },
        )
        .unwrap();

    advance_to_discard(&mut state, 0);
    let drawn = state.last_drawn_tile().unwrap();
    let discards: Vec<Action> = state
        .legal_actions_for(0)
        .into_iter()
        .filter(|a| matches!(a, Action::Discard(_) | Action::Riichi { .. }))
        .collect();
    assert_eq!(discards, vec![Action::Discard(drawn)]);
    let other = state
        .hand(0)
        .concealed()
        .tiles()
        .iter()
        .copied()
        .find(|t| *t != drawn)
        .unwrap();
    assert!(state.apply(0, Action::Discard(other)).is_err());
}

#[test]
fn illegal_tsumo_is_rejected() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(35));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_concealed(0, tiles("1m 4m 7m 1p 4p 7p 1s 4s 7s E S W N wd"));
    assert!(matches!(
        state.apply(0, Action::Tsumo),
        Err(Error::CannotWin)
    ));
}

#[test]
fn double_ron_bonus_goes_to_the_closest_winner() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(24));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_honba_for_test(1);
    state.set_table_riichi_sticks_for_test(1);
    state.set_concealed(1, tenpai_waiting_on_p2());
    state.set_concealed(3, tenpai_waiting_on_p2());
    // Not the first go-around, so neither winner gets renhou.
    state.live_draws = [1; 4];
    let before = *state.scores();

    dealer_discards(&mut state, Tile::pin(2));
    state.apply(1, Action::Ron).unwrap();
    state.apply(2, Action::Pass).unwrap();
    state.apply(3, Action::Ron).unwrap();

    let delta: Vec<i32> = (0..4).map(|s| state.scores()[s] - before[s]).collect();
    // Seat 1 (next after the discarder) also takes 300 honba and the 1000 stick.
    assert_eq!(delta[1] - delta[3], 1_300);
    assert_eq!(delta.iter().sum::<i32>(), 1_000);
    assert_eq!(state.table_riichi_sticks(), 0);
}

#[test]
fn exhaustive_draw_with_four_tenpai_pays_nothing() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(36));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config.clone());
    for seat in 0..4 {
        state.set_concealed(seat, tenpai_waiting_on_p2());
    }
    let profile = RulesRegistry::get(config.profile).unwrap();
    assert_eq!(profile.score_exhaustive_draw(&state, &config), [0; 4]);
}

#[test]
fn kuikae_bans_swap_discards_after_chi_and_pon() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(41));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    // Seat 1 chis 3m with 4m 5m: 3m (called) and 6m (other end) are banned.
    state.set_concealed(1, tiles("4m 5m 3m 6m 1p 2p 3p 7s 8s 9s E E S"));
    dealer_discards(&mut state, Tile::man(3));
    let chi = Action::Chi {
        tiles: [Tile::man(3), Tile::man(4), Tile::man(5)],
    };
    assert!(state.legal_actions_for(1).contains(&chi));
    for seat in 2..4 {
        state.apply(seat, Action::Pass).unwrap();
    }
    state.apply(1, chi).unwrap();
    let legal = state.legal_actions_for(1);
    assert!(!legal.contains(&Action::Discard(Tile::man(3))));
    assert!(!legal.contains(&Action::Discard(Tile::man(6))));
    assert!(legal.contains(&Action::Discard(Tile::pin(1))));
    assert!(matches!(
        state.apply(1, Action::Discard(Tile::man(6))),
        Err(Error::IllegalAction { .. })
    ));
    state.apply(1, Action::Discard(Tile::pin(1))).unwrap();
}

#[test]
fn kuikae_blocks_calls_that_leave_only_banned_discards() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(42));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    // Chi 7m-8m on 9m would leave only 6m and 9m, both banned by kuikae.
    state.set_concealed(1, tiles("7m 8m 6m 9m"));
    dealer_discards(&mut state, Tile::man(9));
    let legal = state.legal_actions_for(1);
    assert!(
        !legal.iter().any(|a| matches!(a, Action::Chi { .. })),
        "{legal:?}"
    );
}

#[test]
fn no_kan_straight_after_pon() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(43));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_concealed(2, tiles("5m 5m 1p 1p 1p 1p 3s 4s 5s 7s 8s 9s E"));
    dealer_discards(&mut state, Tile::man(5));
    for seat in [1, 3] {
        state.apply(seat, Action::Pass).unwrap();
    }
    state.apply(2, Action::Pon).unwrap();
    let legal = state.legal_actions_for(2);
    assert!(
        !legal.iter().any(|a| matches!(a, Action::Kan(_))),
        "{legal:?}"
    );
}

#[test]
fn nine_terminals_on_any_first_uninterrupted_turn() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(44));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_concealed(1, tiles("1m 9m 1p 9p 1s 9s E S W wd 2m 3m 4m"));
    dealer_discards(&mut state, Tile::man(5));
    for seat in 1..4 {
        if state.legal_actions_for(seat).contains(&Action::Pass) {
            state.apply(seat, Action::Pass).unwrap();
        }
    }
    state.apply(1, Action::Draw).unwrap();
    assert!(
        state
            .legal_actions_for(1)
            .contains(&Action::AbortiveNineTerminals)
    );

    let events = state.apply(1, Action::AbortiveNineTerminals).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, crate::event::Event::AbortiveDraw { .. }))
    );
}

#[test]
fn four_kans_by_one_player_do_not_abort() {
    use crate::hand::Meld;
    let config = RulesConfig::standard();
    let profile = crate::rules::RulesRegistry::get(config.profile).unwrap();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(45));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config.clone());
    for _ in 0..4 {
        state.wall_mut().apply_kan().unwrap();
    }
    let kan = |t: Tile| Meld::closed_kan([t; 4]).unwrap();
    let all_four = vec![
        kan(Tile::man(1)),
        kan(Tile::pin(2)),
        kan(Tile::sou(3)),
        kan(Tile::man(4)),
    ];
    state.set_hand(
        1,
        Hand::new(crate::hand::Concealed::from_tiles(tiles("E")), all_four).unwrap(),
    );
    let trigger = crate::game::AbortiveTrigger::KanDeclared;
    assert_eq!(profile.detect_abortive(&state, &config, trigger), None);

    let split = vec![kan(Tile::man(1)), kan(Tile::pin(2)), kan(Tile::sou(3))];
    state.set_hand(
        1,
        Hand::new(
            crate::hand::Concealed::from_tiles(tiles("E 5m 6m 7m")),
            split,
        )
        .unwrap(),
    );
    state.set_hand(
        2,
        Hand::new(
            crate::hand::Concealed::from_tiles(tiles("1s 2s 3s 4s 5s 6s 7s 8s 9s S")),
            vec![kan(Tile::man(4))],
        )
        .unwrap(),
    );
    assert_eq!(
        profile.detect_abortive(&state, &config, trigger),
        Some(crate::game::AbortiveDrawKind::FourKongs)
    );
}

#[test]
fn four_winds_needs_an_uninterrupted_go_around() {
    let config = RulesConfig::standard();
    let profile = crate::rules::RulesRegistry::get(config.profile).unwrap();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(46));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config.clone());
    let east = Tile::wind(crate::tile::Wind::East);
    state.first_discards = [Some(east); 4];
    let trigger = crate::game::AbortiveTrigger::FirstDiscard {
        seat: 3,
        tile: east,
    };
    assert_eq!(
        profile.detect_abortive(&state, &config, trigger),
        Some(crate::game::AbortiveDrawKind::FourWinds)
    );
    state.calls_made = true;
    assert_eq!(profile.detect_abortive(&state, &config, trigger), None);
}
