use crate::hand::{Concealed, Hand};
use crate::rules::standard::win;
use crate::rules::{RulesConfig, RulesProfileId, RulesRegistry, WinContext, WinTimingFlags};
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
    state.is_dealer_first_turn = false;
    state.set_concealed(winner, hand.concealed().tiles().to_vec());
    state
}

#[test]
fn tenpai_after_discard_keeps_wait() {
    let waiting = vec![
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
    ];
    let mut after_draw = waiting.clone();
    after_draw.push(Tile::pin(2));
    let hand = Hand::new(Concealed::from_tiles(after_draw), vec![]).unwrap();

    assert!(win::is_tenpai_after_discard(&hand, Tile::pin(2)));
    assert!(!win::is_tenpai_after_discard(&hand, Tile::man(9)));
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
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(5),
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
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(5),
            Tile::pin(2),
            Tile::pin(2),
        ]),
        vec![],
    )
    .unwrap();

    let state = test_state(1, &hand);
    let ctx = WinContext::new(
        &state,
        1,
        WinType::Tsumo,
        Tile::pin(2),
        WinTimingFlags::default(),
    );

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
    let ctx = WinContext::new(
        &state,
        0,
        WinType::Ron { from: 2 },
        Tile::wind(crate::tile::Wind::East),
        WinTimingFlags::default(),
    );

    let result = profile.score_win(&ctx, &config);
    assert!(result.yaku.contains(&crate::scoring::Yaku::Yakuhai));
    assert!(result.deltas[0] > 0);
    assert!(result.deltas[2] < 0);
}

#[test]
fn pinfu_menzen_tsumo_scores_without_tanyao() {
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();

    let hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::man(1),
            Tile::man(2),
            Tile::man(3),
            Tile::man(5),
            Tile::man(6),
            Tile::man(7),
            Tile::pin(3),
            Tile::pin(4),
            Tile::pin(5),
            Tile::pin(8),
            Tile::pin(8),
            Tile::sou(2),
            Tile::sou(3),
            Tile::sou(4),
        ]),
        vec![],
    )
    .unwrap();

    let state = test_state(0, &hand);
    let ctx = WinContext::new(
        &state,
        0,
        WinType::Tsumo,
        Tile::sou(4),
        WinTimingFlags::default(),
    );

    let result = profile.score_win(&ctx, &config);
    assert!(result.yaku.contains(&crate::scoring::Yaku::Pinfu));
    assert!(result.yaku.contains(&crate::scoring::Yaku::MenzenTsumo));
    assert!(!result.yaku.contains(&crate::scoring::Yaku::Tanyao));
    assert!(result.han >= 2);
}

#[test]
fn dealer_tsumo_earns_more_than_child_tsumo() {
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();

    let tiles = vec![
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
        Tile::sou(5),
        Tile::pin(2),
        Tile::pin(2),
    ];
    let dealer_hand = Hand::new(Concealed::from_tiles(tiles.clone()), vec![]).unwrap();
    let child_hand = Hand::new(Concealed::from_tiles(tiles), vec![]).unwrap();

    let dealer_state = test_state(0, &dealer_hand);
    let child_state = test_state(1, &child_hand);

    let dealer_ctx = WinContext::new(
        &dealer_state,
        0,
        WinType::Tsumo,
        Tile::pin(2),
        WinTimingFlags::default(),
    );
    let child_ctx = WinContext::new(
        &child_state,
        1,
        WinType::Tsumo,
        Tile::pin(2),
        WinTimingFlags::default(),
    );

    let dealer_score = profile.score_win(&dealer_ctx, &config);
    let child_score = profile.score_win(&child_ctx, &config);
    // Same hand: dealer tsumo collects 2× base from each child; ko tsumo collects less total.
    assert!(dealer_score.deltas[0] > child_score.deltas[1]);
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
