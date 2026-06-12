use crate::action::Action;
use crate::agent::{Agent, PlayerView};
use crate::game::Game;
use crate::rules::RulesConfig;
use crate::tile::Tile;

#[test]
fn player_view_hides_opponent_concealed_tiles() {
    let game = Game::new(RulesConfig::standard(), 1).unwrap();
    let view = PlayerView::from_game(&game, 0);

    assert!(!view.own_concealed.is_empty());
    for (seat, seat_view) in view.seats.iter().enumerate() {
        if seat != 0 {
            assert_eq!(
                seat_view.concealed_count,
                game.hand().hand(seat).concealed().len()
            );
        }
    }
}

#[test]
fn step_applies_agent_action() {
    let mut game = Game::new(RulesConfig::standard(), 2).unwrap();
    let dealer = game.dealer();
    let discard = game.hand().hand(dealer).concealed().tiles()[0];

    #[derive(Clone, Copy)]
    struct PickFirst;

    impl Agent for PickFirst {
        fn decide(&mut self, _: &PlayerView, legal: &[Action]) -> Action {
            legal[0]
        }
    }

    let mut agents = [PickFirst, PickFirst, PickFirst, PickFirst];
    let result = game.step(&mut agents).unwrap().expect("step taken");
    assert_eq!(result.seat, dealer);
    assert_eq!(result.action, Action::Discard(discard));
}

#[test]
fn drive_hand_via_actions() {
    let mut game = Game::new(RulesConfig::standard(), 3).unwrap();
    let winner = game.dealer();
    game.hand_mut().set_concealed(
        winner,
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
            Tile::sou(9),
            Tile::sou(9),
            Tile::sou(9),
            Tile::pin(2),
            Tile::pin(2),
        ],
    );
    game.hand_mut().last_draw = Some(Tile::pin(2));

    let events = game.apply_action(winner, Action::Tsumo).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, crate::event::Event::Won { .. }))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, crate::event::Event::HandStarted { .. }))
    );
}
