use crate::rules::{RulesConfig, RulesProfileId, RulesRegistry};
use crate::state::HandState;
use crate::tile::Tile;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

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
