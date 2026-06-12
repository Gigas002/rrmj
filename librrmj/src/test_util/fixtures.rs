use crate::action::Action;
use crate::event::Event;
use crate::game::Game;
use crate::rules::RulesConfig;
use crate::state::HandPhase;
use crate::tile::Tile;

/// Concealed tanyao winning hand (14 tiles) used across match/replay/state tests.
pub fn winning_tanyao_tiles() -> Vec<Tile> {
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
    ]
}

/// Tenpai on `2p` ron (13 tiles).
pub fn tenpai_waiting_on_p2() -> Vec<Tile> {
    let mut hand = winning_tanyao_tiles();
    hand.pop();
    hand
}

/// Tenpai after drawing the winning tile (tile is in concealed, as after `apply_draw`).
pub fn tenpai_after_draw_p2() -> Vec<Tile> {
    let mut hand = tenpai_waiting_on_p2();
    hand.push(Tile::pin(2));
    hand
}

pub fn force_tsumo_win(game: &mut Game, seat: usize) {
    let hand = game.hand_mut();
    hand.set_concealed(seat, winning_tanyao_tiles());
    hand.last_draw = Some(Tile::pin(2));
}

pub fn force_ron_setup(game: &mut Game, winner: usize) {
    let dealer = game.dealer();
    let hand = game.hand_mut();
    hand.set_concealed(winner, tenpai_waiting_on_p2());
    let mut dealer_hand = hand.hand(dealer).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    hand.set_concealed(dealer, dealer_hand);
    hand.apply(dealer, Action::Discard(Tile::pin(2)))
        .expect("dealer discard for ron setup");
}

pub fn resolve_pending_reactions(game: &mut Game, winner: usize) -> Vec<Event> {
    let mut events = Vec::new();
    while game.hand().phase() == HandPhase::Reaction {
        let seat = game.hand().pending_reaction_seat().expect("reaction seat");
        let action = if seat == winner {
            Action::Ron
        } else {
            Action::Pass
        };
        events.extend(game.apply_action(seat, action).expect("reaction action"));
    }
    events
}

/// Sets up and resolves a ron win through the match API (advances match state).
pub fn force_ron_win(game: &mut Game, winner: usize) -> Vec<Event> {
    force_ron_setup(game, winner);
    resolve_pending_reactions(game, winner)
}

pub fn play_tsumo_hand(seed: u64) -> Game {
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    let winner = game.dealer();
    force_tsumo_win(&mut game, winner);
    game.apply_action(winner, Action::Tsumo).unwrap();
    game
}
