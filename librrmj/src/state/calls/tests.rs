use super::{ChiPosition, chi_actions, chi_position, kakan_options};
use crate::action::Action;
use crate::hand::{Concealed, Hand, Meld};
use crate::tile::Tile;

#[test]
fn chi_left_middle_right_actions() {
    let called = Tile::pin(5);
    let concealed = Concealed::from_tiles(vec![
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(6),
        Tile::pin(7),
        Tile::pin(8),
        Tile::pin(9),
        Tile::man(1),
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::man(5),
        Tile::man(6),
        Tile::man(7),
    ]);

    let actions = chi_actions(&concealed, called);
    assert_eq!(actions.len(), 3);

    let positions: Vec<ChiPosition> = actions
        .iter()
        .filter_map(|action| match action {
            Action::Chi { tiles } => chi_position(called, *tiles),
            _ => None,
        })
        .collect();
    assert!(positions.contains(&ChiPosition::Left));
    assert!(positions.contains(&ChiPosition::Middle));
    assert!(positions.contains(&ChiPosition::Right));
}

#[test]
fn kakan_option_on_open_pon_with_fourth_tile() {
    let pon = Meld::pon([Tile::sou(5), Tile::sou(5), Tile::sou(5)], Tile::sou(5)).unwrap();
    let hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::sou(5),
            Tile::man(1),
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::man(6),
            Tile::man(7),
            Tile::man(8),
            Tile::man(9),
            Tile::pin(1),
            Tile::pin(2),
        ]),
        vec![pon],
    )
    .unwrap();

    assert_eq!(kakan_options(&hand), vec![0]);
}
