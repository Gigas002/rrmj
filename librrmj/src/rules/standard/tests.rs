use crate::hand::{Concealed, Hand};
use crate::rules::standard::win;
use crate::rules::{RulesConfig, RulesProfileId, RulesRegistry, WinContext};
use crate::scoring::WinType;
use crate::state::HandState;
use crate::tile::Tile;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn test_state(winner: usize, hand: &Hand) -> HandState {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(1));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_concealed(winner, hand.concealed().tiles().to_vec());
    state
}

#[test]
fn tanyao_fixture_is_winning() {
    let hand = Hand::new(
        Concealed::from_tiles(vec![
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
        ]),
        vec![],
    )
    .unwrap();
    assert!(win::is_winning_hand(&hand, Some(Tile::pin(2))));
}

#[test]
fn tanyao_tsumo_scores() {
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();

    let hand = Hand::new(
        Concealed::from_tiles(vec![
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
        ]),
        vec![],
    )
    .unwrap();

    let state = test_state(1, &hand);
    let ctx = WinContext {
        state: &state,
        winner: 1,
        win_type: WinType::Tsumo,
        win_tile: Tile::pin(2),
    };

    assert!(profile.can_win(&ctx, &config));

    let result = profile.score_win(&ctx, &config);
    assert!(result.han >= 2);
    assert!(result.deltas[1] > 0);
}

#[test]
fn yakuhai_ron_scores() {
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();

    let hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::pin(5),
            Tile::pin(6),
            Tile::pin(7),
            Tile::sou(6),
            Tile::sou(7),
            Tile::sou(8),
            Tile::wind(crate::tile::Wind::East),
            Tile::wind(crate::tile::Wind::East),
            Tile::pin(2),
            Tile::pin(3),
        ]),
        vec![],
    )
    .unwrap();

    let state = test_state(0, &hand);
    let ctx = WinContext {
        state: &state,
        winner: 0,
        win_type: WinType::Ron { from: 2 },
        win_tile: Tile::wind(crate::tile::Wind::East),
    };

    let result = profile.score_win(&ctx, &config);
    assert!(result.yaku.contains(&crate::scoring::Yaku::Yakuhai));
    assert!(result.deltas[0] > 0);
    assert!(result.deltas[2] < 0);
}

// --- abortive draws ---

#[test]
fn nine_terminals_detected_on_dealer_first_turn() {
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(50));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.set_concealed(
        0,
        vec![
            Tile::man(1),
            Tile::man(9),
            Tile::pin(1),
            Tile::pin(9),
            Tile::sou(1),
            Tile::sou(9),
            Tile::wind(crate::tile::Wind::East),
            Tile::wind(crate::tile::Wind::South),
            Tile::wind(crate::tile::Wind::West),
            Tile::dragon(crate::tile::Dragon::White),
            Tile::dragon(crate::tile::Dragon::Green),
            Tile::dragon(crate::tile::Dragon::Red),
            Tile::man(2),
            Tile::man(3),
        ],
    );

    assert!(
        state
            .legal_actions_for(0)
            .contains(&crate::action::Action::AbortiveNineTerminals)
    );
    let events = state
        .apply(0, crate::action::Action::AbortiveNineTerminals)
        .unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, crate::event::Event::AbortiveDraw { .. }))
    );
    assert!(state.is_ended());
    let _ = profile;
}
