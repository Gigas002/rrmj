use crate::action::Action;
use crate::agent::Agent;
use crate::agent::PlayerView;
use crate::ai::MediumAgent;
use crate::ai::shanten;
use crate::game::RoundWind;
use crate::hand::{Concealed, Hand};
use crate::state::{HandPhase, SEAT_COUNT};
use crate::tile::Tile;

fn empty_view(concealed: Vec<Tile>, phase: HandPhase) -> PlayerView {
    PlayerView {
        seat: 0,
        dealer: 0,
        phase,
        current_actor: 0,
        round_wind: RoundWind::East,
        kyoku: 1,
        honba: 0,
        scores: [25_000; 4],
        own_concealed: concealed,
        seats: std::array::from_fn(|_| crate::agent::SeatView {
            melds: Vec::new(),
            discards: Vec::new(),
            riichi: false,
            concealed_count: 0,
        }),
        dora_indicators: Vec::new(),
        table_riichi_sticks: 0,
        turn: crate::agent::TurnContext::idle(),
    }
}

#[test]
fn prefers_discard_that_reduces_shanten() {
    let concealed = vec![
        Tile::pin(2),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::pin(5),
        Tile::pin(6),
        Tile::pin(7),
        Tile::pin(8),
        Tile::sou(2),
        Tile::sou(3),
        Tile::sou(4),
        Tile::sou(5),
        Tile::sou(5),
        Tile::wind(crate::tile::Wind::East),
    ];
    let view = empty_view(concealed.clone(), HandPhase::Discard);
    let legal: Vec<Action> = view
        .own_concealed
        .iter()
        .copied()
        .map(Action::Discard)
        .collect();

    let hand = Hand::new(crate::hand::Concealed::from_tiles(concealed), vec![]).unwrap();
    let baseline = shanten::best_waiting_potential(&hand);

    let mut agent = MediumAgent::new(7);
    let choice = agent.decide(&view, &legal);
    let Action::Discard(tile) = choice else {
        panic!("expected discard, got {choice:?}");
    };
    let after = shanten::hand_without_concealed_tile(&hand, tile).unwrap();
    assert!(shanten::waiting_count(&after) >= baseline);
}

#[test]
fn passes_when_pon_reduces_waiting_potential() {
    let concealed = vec![
        Tile::pin(2),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(6),
        Tile::pin(7),
        Tile::pin(8),
        Tile::sou(2),
        Tile::sou(3),
        Tile::sou(4),
        Tile::sou(5),
        Tile::sou(5),
        Tile::sou(6),
        Tile::sou(7),
    ];
    let hand = Hand::new(Concealed::from_tiles(concealed.clone()), vec![]).unwrap();
    let called = Tile::sou(5);
    let after = crate::ai::common::simulate_call(&hand, Action::Pon, called).unwrap();
    assert!(
        shanten::best_waiting_potential(&after) < shanten::best_waiting_potential(&hand),
        "fixture should worsen after pon"
    );

    let mut view = empty_view(concealed, HandPhase::Reaction);
    view.turn = crate::agent::TurnContext::reaction(crate::agent::PendingCall {
        discarder: 1,
        tile: called,
    });

    let legal = [Action::Pass, Action::Pon];
    let mut agent = MediumAgent::new(11);
    assert_eq!(agent.decide(&view, &legal), Action::Pass);
}

#[test]
fn hand_helpers_build_valid_hands() {
    let hand = Hand::new(Concealed::from_tiles(vec![Tile::man(1); 13]), vec![]).unwrap();
    assert_eq!(hand.total_tiles(), 13);
}

#[test]
fn view_has_four_seats() {
    assert_eq!(SEAT_COUNT, 4);
}
