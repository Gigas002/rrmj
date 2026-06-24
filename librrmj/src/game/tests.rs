use super::{Game, GameLength, RoundWind};
use crate::action::Action;
use crate::error::Error;
use crate::event::Event;
use crate::game::HandOutcome;
use crate::rules::RulesConfig;
use crate::rules::flow::advance_after_hand;
use crate::state::HandPhase;
use crate::tile::Tile;

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
        Tile::sou(9),
        Tile::sou(9),
        Tile::sou(9),
        Tile::pin(2),
        Tile::pin(2),
    ]
}

fn tenpai_waiting_on_p2() -> Vec<Tile> {
    let mut hand = winning_tanyao_tiles();
    hand.pop();
    hand
}

fn force_tsumo_win(game: &mut Game, seat: usize) {
    let hand = game.hand_mut();
    hand.set_concealed(seat, winning_tanyao_tiles());
    hand.last_draw = Some(Tile::pin(2));
}

fn force_ron_setup(game: &mut Game, winner: usize) {
    let dealer = game.dealer();
    let hand = game.hand_mut();
    hand.set_concealed(winner, tenpai_waiting_on_p2());
    let mut dealer_hand = hand.hand(dealer).concealed().tiles().to_vec();
    dealer_hand[0] = Tile::pin(2);
    hand.set_concealed(dealer, dealer_hand);
    hand.apply(dealer, Action::Discard(Tile::pin(2)))
        .expect("dealer discard for ron setup");
}

fn resolve_pending_reactions(game: &mut Game, winner: usize) -> Vec<Event> {
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

fn force_ron_win(game: &mut Game, winner: usize) -> Vec<Event> {
    force_ron_setup(game, winner);
    resolve_pending_reactions(game, winner)
}

impl Game {
    fn finish_hand_for_test(&mut self) -> Result<Vec<Event>, Error> {
        let events = self.finish_hand()?;
        self.record_events(events.clone());
        Ok(events)
    }
}

// --- flow helpers ---

#[test]
fn dealer_renchan_increments_honba() {
    let (dealer, honba, round, kyoku) = advance_after_hand(
        0,
        0,
        RoundWind::East,
        1,
        HandOutcome::Win { winners: vec![0] },
        false,
    );
    assert_eq!(dealer, 0);
    assert_eq!(honba, 1);
    assert_eq!(round, RoundWind::East);
    assert_eq!(kyoku, 1);
}

#[test]
fn non_dealer_win_rotates_dealer_and_advances_kyoku() {
    let (dealer, honba, round, kyoku) = advance_after_hand(
        0,
        2,
        RoundWind::East,
        1,
        HandOutcome::Win { winners: vec![1] },
        false,
    );
    assert_eq!(dealer, 1);
    assert_eq!(honba, 0);
    assert_eq!(round, RoundWind::East);
    assert_eq!(kyoku, 2);
}

#[test]
fn east_four_to_south_one() {
    let (dealer, honba, round, kyoku) = advance_after_hand(
        3,
        0,
        RoundWind::East,
        4,
        HandOutcome::Win { winners: vec![1] },
        false,
    );
    assert_eq!(dealer, 0);
    assert_eq!(honba, 0);
    assert_eq!(round, RoundWind::South);
    assert_eq!(kyoku, 1);
}

#[test]
fn nine_terminals_abortive_keeps_dealer_and_honba() {
    use crate::game::AbortiveDrawKind;
    let (dealer, honba, round, kyoku) = advance_after_hand(
        2,
        3,
        RoundWind::South,
        2,
        HandOutcome::AbortiveDraw(AbortiveDrawKind::NineTerminals),
        false,
    );
    assert_eq!(dealer, 2);
    assert_eq!(honba, 3);
    assert_eq!(round, RoundWind::South);
    assert_eq!(kyoku, 2);
}

// --- game integration ---

#[test]
fn east_only_match_ends_after_four_kyoku() {
    let mut config = RulesConfig::standard();
    config.game_length = GameLength::EastOnly;

    let mut game = Game::new(config, 100).unwrap();
    assert_eq!(game.round_wind(), RoundWind::East);
    assert_eq!(game.kyoku(), 1);

    for expected_kyoku in [2u8, 3, 4] {
        let winner = (game.dealer() + 1) % 4;
        let events = force_ron_win(&mut game, winner);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::HandStarted { .. }))
        );
        assert!(!game.is_ended());
        assert_eq!(game.kyoku(), expected_kyoku);
    }

    let winner = (game.dealer() + 1) % 4;
    let events = force_ron_win(&mut game, winner);
    assert!(game.is_ended());
    assert!(events.iter().any(|e| matches!(e, Event::GameEnded { .. })));
}

#[test]
fn honba_carries_into_next_hand() {
    let config = RulesConfig::standard();
    let mut game = Game::new(config, 200).unwrap();

    let dealer = game.dealer();
    force_tsumo_win(&mut game, dealer);
    game.apply_action(dealer, Action::Tsumo).unwrap();

    assert_eq!(game.honba(), 1);
    assert_eq!(game.hand().honba(), 1);
}

#[test]
fn scores_carry_between_hands() {
    let config = RulesConfig::standard();
    let mut game = Game::new(config, 300).unwrap();
    let starting = game.config().starting_points;

    let dealer = game.dealer();
    force_tsumo_win(&mut game, dealer);
    game.apply_action(dealer, Action::Tsumo).unwrap();

    assert!(game.scores()[dealer] > starting);
    assert_eq!(game.hand().scores()[dealer], game.scores()[dealer]);
}

#[test]
fn target_score_ends_match_early() {
    let mut config = RulesConfig::standard();
    config.game_length = GameLength::EastOnly;
    config.target_score = Some(26_000);

    let mut game = Game::new(config, 400).unwrap();
    let dealer = game.dealer();
    force_tsumo_win(&mut game, dealer);
    let events = game.apply_action(dealer, Action::Tsumo).unwrap();

    assert!(game.is_ended());
    assert!(events.iter().any(|e| matches!(e, Event::GameEnded { .. })));
    assert!(game.scores().iter().any(|&s| s >= 26_000));
}

#[test]
fn hanchan_runs_eight_kyoku() {
    let mut config = RulesConfig::standard();
    config.game_length = GameLength::Hanchan;

    let mut game = Game::new(config, 600).unwrap();
    let mut hands_played = 0u8;

    while !game.is_ended() && hands_played < 16 {
        let winner = (game.dealer() + 1) % 4;
        force_ron_win(&mut game, winner);
        hands_played += 1;
    }

    assert!(game.is_ended());
    assert_eq!(hands_played, 8);
}

#[test]
fn exhaustive_draw_advances_match() {
    let mut config = RulesConfig::standard();
    config.game_length = GameLength::EastOnly;

    let mut game = Game::new(config, 500).unwrap();
    let kyoku_before = game.kyoku();

    game.hand_mut()
        .play_out_discards(|state, seat| state.hand(seat).concealed().tiles()[0])
        .unwrap();

    let events = game.finish_hand_for_test().unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::HandStarted { .. }))
    );
    assert!(!game.is_ended());
    assert_eq!(game.kyoku(), kyoku_before + 1);
}
