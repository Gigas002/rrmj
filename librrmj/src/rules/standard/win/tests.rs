use crate::hand::{Concealed, Hand};
use crate::rules::standard::win;
use crate::tile::Tile;

#[test]
fn tenpai_after_discard_keeps_wait() {
    let waiting = vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(9),
        Tile::sou(9),
        Tile::sou(9),
        Tile::pin(2),
    ];
    let mut after_draw = waiting.clone();
    after_draw.push(Tile::pin(2));
    let hand = Hand::new(Concealed::from_tiles(after_draw), vec![]).unwrap();

    assert!(win::is_tenpai_after_discard(&hand, Tile::pin(2)));
    assert!(!win::is_tenpai_after_discard(&hand, Tile::man(9)));
}

#[test]
fn tanyao_fixture_is_winning() {
    let hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::pin(3),
            Tile::pin(4),
            Tile::pin(5),
            Tile::sou(6),
            Tile::sou(7),
            Tile::sou(8),
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(5),
            Tile::pin(2),
            Tile::pin(2),
        ]),
        vec![],
    )
    .unwrap();
    assert!(win::is_winning_hand(&hand, Some(Tile::pin(2))));
}

#[test]
fn open_hand_tsumo_does_not_double_count_win_tile() {
    use crate::hand::Meld;
    use crate::tile::{Dragon, Suit};

    let hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::sou(2),
            Tile::sou(2),
            Tile::red_five(Suit::Sou),
            Tile::sou(6),
            Tile::sou(7),
        ]),
        vec![
            Meld::pon([Tile::dragon(Dragon::White); 3], Tile::dragon(Dragon::White)).unwrap(),
            Meld::chi([Tile::pin(5), Tile::pin(6), Tile::pin(7)], Tile::pin(6)).unwrap(),
            Meld::chi(
                [Tile::red_five(Suit::Man), Tile::man(6), Tile::man(7)],
                Tile::red_five(Suit::Man),
            )
            .unwrap(),
        ],
    )
    .unwrap();

    assert!(win::is_winning_hand(&hand, Some(Tile::sou(2))));
}
