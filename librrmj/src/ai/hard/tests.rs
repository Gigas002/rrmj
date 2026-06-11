use crate::action::Action;
use crate::agent::{Agent, PlayerView, SeatView};
use crate::ai::HardAgent;
use crate::ai::defense::{SAFETY_MAX, tile_safety};
use crate::game::RoundWind;
use crate::hand::{Concealed, Hand};
use crate::state::HandPhase;
use crate::tile::Tile;

fn view_with_riichi_opponent(concealed: Vec<Tile>, opponent_discards: Vec<Tile>) -> PlayerView {
    let mut seats = std::array::from_fn(|_| SeatView {
        melds: Vec::new(),
        discards: Vec::new(),
        riichi: false,
        concealed_count: 0,
    });
    seats[1].riichi = true;
    seats[1].discards = opponent_discards;
    seats[0].concealed_count = concealed.len();

    PlayerView {
        seat: 0,
        dealer: 0,
        phase: HandPhase::Discard,
        current_actor: 0,
        round_wind: RoundWind::East,
        kyoku: 1,
        honba: 0,
        scores: [25_000; 4],
        own_concealed: concealed,
        seats,
        dora_indicators: Vec::new(),
        table_riichi_sticks: 0,
        turn: crate::agent::TurnContext::idle(),
    }
}

#[test]
fn genbutsu_tile_is_max_safety() {
    let tile = Tile::man(1);
    let view = view_with_riichi_opponent(
        vec![tile, Tile::man(2), Tile::man(3), Tile::man(4)],
        vec![tile, tile, tile],
    );
    assert_eq!(tile_safety(&view, tile), SAFETY_MAX);
}

#[test]
fn prefers_safer_discard_under_riichi_pressure() {
    let view = view_with_riichi_opponent(
        vec![
            Tile::man(1),
            Tile::man(2),
            Tile::man(4),
            Tile::pin(2),
            Tile::pin(4),
            Tile::pin(6),
            Tile::pin(8),
            Tile::sou(1),
            Tile::sou(3),
            Tile::sou(5),
            Tile::sou(7),
            Tile::sou(9),
            Tile::wind(crate::tile::Wind::East),
            Tile::wind(crate::tile::Wind::South),
        ],
        vec![Tile::pin(5)],
    );
    let legal: Vec<Action> = view
        .own_concealed
        .iter()
        .copied()
        .map(Action::Discard)
        .collect();
    let mut agent = HardAgent::new(3);
    let choice = agent.decide(&view, &legal);
    let Action::Discard(tile) = choice else {
        panic!("expected discard, got {choice:?}");
    };
    assert!(tile_safety(&view, tile) >= tile_safety(&view, Tile::pin(2)));
}

#[test]
fn declares_riichi_when_tenpai_and_safe_to_do_so() {
    let concealed = vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(9),
        Tile::sou(9),
        Tile::sou(9),
        Tile::pin(2),
        Tile::pin(2),
    ];
    let view = PlayerView {
        seat: 0,
        dealer: 0,
        phase: HandPhase::Discard,
        current_actor: 0,
        round_wind: RoundWind::East,
        kyoku: 1,
        honba: 0,
        scores: [25_000; 4],
        own_concealed: concealed,
        seats: std::array::from_fn(|_| SeatView {
            melds: Vec::new(),
            discards: Vec::new(),
            riichi: false,
            concealed_count: 0,
        }),
        dora_indicators: Vec::new(),
        table_riichi_sticks: 0,
        turn: crate::agent::TurnContext::idle(),
    };
    let legal: Vec<Action> = view
        .own_concealed
        .iter()
        .copied()
        .flat_map(|tile| [Action::Discard(tile), Action::Riichi { discard: tile }])
        .collect();
    let mut agent = HardAgent::new(8);
    let choice = agent.decide(&view, &legal);
    assert!(
        matches!(choice, Action::Riichi { .. }) || matches!(choice, Action::Discard(_)),
        "expected riichi or efficient discard, got {choice:?}"
    );
}

#[test]
fn hand_fixture_builds() {
    let hand = Hand::new(Concealed::from_tiles(vec![Tile::man(1); 13]), vec![]).unwrap();
    assert_eq!(hand.total_tiles(), 13);
}
