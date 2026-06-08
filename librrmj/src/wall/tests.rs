use rand::SeedableRng;
use rand::rngs::StdRng;

use super::{
    DEAD_WALL_SIZE, INITIAL_DEAL_SIZE, LIVE_WALL_AFTER_DEAL, LIVE_WALL_AFTER_SPLIT, WALL_SIZE, Wall,
};
use crate::hand::{DEALER_HAND_SIZE, NON_DEALER_HAND_SIZE};
use crate::rules::RulesConfig;
use crate::tile::{Tile, standard_set};

#[test]
fn standard_set_and_wall_partition_sizes() {
    let rules = RulesConfig::standard();
    let rng = StdRng::seed_from_u64(1);
    let wall = Wall::new(&rules, rng);

    assert_eq!(wall.live_remaining(), LIVE_WALL_AFTER_SPLIT);
    assert_eq!(wall.dead_wall().len(), DEAD_WALL_SIZE);
    assert_eq!(wall.live_remaining() + wall.dead_wall().len(), WALL_SIZE);
}

#[test]
fn deal_distributes_correct_tile_counts() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(42));
    let dealer = 2;

    let deal = wall.deal(dealer).unwrap();

    assert_eq!(deal.dealer, dealer);
    assert_eq!(deal.live_remaining, LIVE_WALL_AFTER_DEAL);
    assert_eq!(wall.live_drawn(), INITIAL_DEAL_SIZE);

    for seat in 0..4 {
        let expected = if seat == dealer {
            DEALER_HAND_SIZE
        } else {
            NON_DEALER_HAND_SIZE
        };
        assert_eq!(deal.hand(seat).total_tiles(), expected);
    }
}

#[test]
fn deal_conserves_all_tiles() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(7));
    let deal = wall.deal(0).unwrap();

    let mut dealt: Vec<Tile> = Vec::new();
    for hand in &deal.hands {
        dealt.extend(hand.concealed().tiles());
    }
    dealt.extend(wall.dead_wall());
    dealt.extend(wall.layout().live());

    let mut expected = standard_set(rules.aka_dora);
    dealt.sort();
    expected.sort();
    assert_eq!(dealt, expected);
}

#[test]
fn live_wall_exhaustion_after_max_draws() {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(99));
    wall.deal(0).unwrap();

    for _ in 0..LIVE_WALL_AFTER_DEAL {
        assert!(wall.draw_live().is_ok());
    }
    assert!(matches!(
        wall.draw_live(),
        Err(crate::Error::LiveWallExhausted)
    ));
}

#[test]
fn dora_indicator_is_first_dead_wall_tile() {
    let rules = RulesConfig::standard();
    let wall = Wall::new(&rules, StdRng::seed_from_u64(3));
    assert_eq!(wall.dora_indicator(), wall.dead_wall()[0]);
}
