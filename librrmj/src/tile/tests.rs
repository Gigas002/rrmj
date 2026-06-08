use std::str::FromStr;

use super::{Dragon, Suit, Tile, Wind, standard_set};

#[test]
fn standard_set_has_136_tiles() {
    assert_eq!(standard_set(true).len(), 136);
    assert_eq!(standard_set(false).len(), 136);
}

#[test]
fn aka_dora_replaces_one_five_per_suit() {
    let tiles = standard_set(true);
    let red_fives = tiles.iter().filter(|t| t.is_red()).count();
    assert_eq!(red_fives, 3);

    let normal_fives = tiles
        .iter()
        .filter(|t| t.rank() == Some(5) && !t.is_red())
        .count();
    assert_eq!(normal_fives, 9);
}

#[test]
fn sort_order_numbered_then_honors() {
    let mut tiles = [
        Tile::dragon(Dragon::Red),
        Tile::man(1),
        Tile::wind(Wind::East),
        Tile::pin(9),
        Tile::sou(3),
        Tile::red_five(Suit::Man),
        Tile::man(5),
    ];
    tiles.sort();

    let labels: Vec<String> = tiles.iter().map(|t| t.to_string()).collect();
    assert_eq!(labels, vec!["1m", "5m", "5mr", "9p", "3s", "E", "rd"]);
}

#[test]
fn display_and_parse_round_trip() {
    let samples = ["1m", "9p", "5sr", "E", "W", "wd", "gd", "rd"];

    for label in samples {
        let tile = Tile::from_str(label).expect(label);
        assert_eq!(tile.to_string(), label);
    }
}

#[test]
fn invalid_tile_strings_are_rejected() {
    assert!(Tile::from_str("0m").is_err());
    assert!(Tile::from_str("5mr").is_ok());
    assert!(Tile::from_str("3mr").is_err());
    assert!(Tile::from_str("wd").is_ok());
    assert!(Tile::from_str("wdr").is_err());
}
