use librrmj::tile::Tile;

use super::{drawn_hand_index, picker_index_to_hand_index};

#[test]
fn picker_index_steps_through_duplicate_tiles() {
    let hand = [
        Tile::man(8),
        Tile::man(8),
        Tile::sou(1),
        Tile::sou(2),
        Tile::sou(3),
    ];
    let discards = hand.to_vec();
    assert_eq!(picker_index_to_hand_index(&hand, &discards, 0), Some(0));
    assert_eq!(picker_index_to_hand_index(&hand, &discards, 1), Some(1));
    assert_eq!(picker_index_to_hand_index(&hand, &discards, 2), Some(2));
}

#[test]
fn drawn_tile_highlights_rightmost_copy() {
    let hand = [Tile::man(8), Tile::man(8), Tile::sou(1)];
    assert_eq!(drawn_hand_index(&hand, Some(Tile::man(8))), Some(1));
}
