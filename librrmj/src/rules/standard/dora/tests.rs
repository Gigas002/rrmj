use crate::rules::standard::dora::dora_tile;
use crate::tile::{Dragon, Tile, Wind};

#[test]
fn dora_tile_advances_numbered_ranks() {
    assert_eq!(dora_tile(Tile::man(5)), Some(Tile::man(6)));
    assert_eq!(dora_tile(Tile::man(9)), Some(Tile::man(1)));
}

#[test]
fn dora_tile_advances_honors() {
    assert_eq!(dora_tile(Tile::wind(Wind::North)), Some(Tile::wind(Wind::East)));
    assert_eq!(
        dora_tile(Tile::dragon(Dragon::Red)),
        Some(Tile::dragon(Dragon::White))
    );
}
