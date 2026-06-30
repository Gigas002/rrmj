use super::*;
use crate::tile::Tile;

#[test]
fn format_includes_discard_wait_and_groups() {
    let decomp = PathDecomposition {
        groups: vec![
            PathGroup {
                tiles: vec![Tile::man(2), Tile::man(3), Tile::man(4)],
                open: false,
            },
            PathGroup {
                tiles: vec![Tile::pin(2), Tile::pin(2)],
                open: false,
            },
        ],
        missing: vec![Tile::sou(2)],
        suggested_discard: Some(Tile::sou(9)),
    };
    let lines = decomp.format_lines();
    assert_eq!(lines[0], "Discard 9s →");
    assert!(lines[1].contains("2m3m4m"));
    assert_eq!(lines[2], "Need +2s");
}
