use rand::SeedableRng;
use rand::rngs::StdRng;

use super::EasyAgent;
use crate::action::Action;
use crate::tile::Tile;

#[test]
fn always_takes_tsumo_when_legal() {
    let mut rng = StdRng::seed_from_u64(1);
    let legal = [
        Action::Discard(Tile::man(1)),
        Action::Tsumo,
        Action::Discard(Tile::man(2)),
    ];
    assert_eq!(EasyAgent::decide_with_rng(&mut rng, &legal), Action::Tsumo);
}

#[test]
fn always_takes_ron_when_legal() {
    let mut rng = StdRng::seed_from_u64(2);
    let legal = [Action::Pass, Action::Ron];
    assert_eq!(EasyAgent::decide_with_rng(&mut rng, &legal), Action::Ron);
}

#[test]
fn random_among_non_win_actions() {
    let mut rng = StdRng::seed_from_u64(99);
    let legal = [
        Action::Pass,
        Action::Discard(Tile::pin(3)),
        Action::Discard(Tile::pin(4)),
    ];
    let choice = EasyAgent::decide_with_rng(&mut rng, &legal);
    assert!(legal.contains(&choice));
    assert!(!matches!(choice, Action::Tsumo | Action::Ron));
}
