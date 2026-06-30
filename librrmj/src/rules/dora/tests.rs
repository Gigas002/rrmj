use super::*;
use crate::tile::Tile;

#[test]
fn indicator_dora_wraps_suit_nine_to_one() {
    assert_eq!(dora_tile(Tile::man(9)), Some(Tile::man(1)));
    assert!(matches_indicator_dora(Tile::man(1), &[Tile::man(9)]));
}

#[test]
fn aka_dora_requires_red_five() {
    assert!(is_aka_dora(Tile::red_five(crate::tile::Suit::Pin), true));
    assert!(!is_aka_dora(Tile::pin(5), true));
}
