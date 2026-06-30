use librrmj::tile::{Suit, Tile};

use super::tile_matches_highlight;

#[test]
fn highlight_matches_red_and_normal_five() {
    let red = Tile::red_five(Suit::Pin);
    let normal = Tile::pin(5);
    assert!(tile_matches_highlight(red, normal));
    assert!(tile_matches_highlight(normal, red));
}

#[test]
fn highlight_distinguishes_different_suits() {
    let pin5 = Tile::pin(5);
    let sou5 = Tile::sou(5);
    assert!(!tile_matches_highlight(pin5, sou5));
}

#[test]
fn highlight_distinguishes_different_ranks() {
    let pin4 = Tile::pin(4);
    let pin5 = Tile::pin(5);
    assert!(!tile_matches_highlight(pin4, pin5));
}
