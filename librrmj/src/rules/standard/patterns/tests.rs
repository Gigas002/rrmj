use crate::hand::{Concealed, Hand};
use crate::rules::standard::patterns;
use crate::rules::{RulesConfig, WinContext, WinTimingFlags};
use crate::scoring::WinType;
use crate::state::HandState;
use crate::tile::Tile;
use crate::wall::Wall;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[test]
fn toitoi_hand_detected_from_all_triplets() {
    let config = RulesConfig::standard();
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(7));
    let deal = wall.deal(0).unwrap();
    let mut state = HandState::from_deal(wall, deal, config);
    state.is_dealer_first_turn = false;
    let hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::man(2),
            Tile::man(2),
            Tile::man(2),
            Tile::pin(3),
            Tile::pin(3),
            Tile::pin(3),
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(8),
            Tile::sou(8),
            Tile::sou(8),
            Tile::pin(7),
            Tile::pin(7),
        ]),
        vec![],
    )
    .unwrap();
    state.set_hand(0, hand);
    let ctx = WinContext::new(
        &state,
        0,
        WinType::Tsumo,
        Tile::pin(7),
        WinTimingFlags::default(),
    );
    assert!(patterns::is_toitoi_hand(&ctx));
}
