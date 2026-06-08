use super::{DEALER_HAND_SIZE, Hand, NON_DEALER_HAND_SIZE, validate_deal_counts};
use crate::hand::{Concealed, Meld};
use crate::tile::Tile;

#[test]
fn concealed_sorts_tiles() {
    let concealed = Concealed::from_tiles(vec![Tile::pin(9), Tile::man(1), Tile::man(3)]);
    assert_eq!(concealed.tiles()[0], Tile::man(1));
    assert_eq!(concealed.tiles()[2], Tile::pin(9));
}

#[test]
fn meld_validates_tile_counts() {
    let tile = Tile::man(1);
    assert!(Meld::chi([tile, Tile::man(2), Tile::man(3)], tile).is_ok());
    assert!(Meld::pon([tile, tile, tile], tile).is_ok());
    assert!(Meld::closed_kan([tile, tile, tile, tile]).is_ok());
    assert!(Meld::added_kan(tile).is_ok());

    assert!(
        Meld::try_new(
            super::meld::MeldKind::Chi,
            vec![tile, Tile::man(2)],
            Some(tile),
        )
        .is_err()
    );
}

#[test]
fn hand_total_includes_melds() {
    let tile = Tile::pin(5);
    let meld = Meld::pon([tile, tile, tile], tile).unwrap();
    let mut concealed = Concealed::empty();
    for _ in 0..10 {
        concealed.push(Tile::man(2));
    }

    let hand = Hand::new(concealed, vec![meld]).unwrap();
    assert_eq!(hand.total_tiles(), 13);
}

#[test]
fn validate_deal_counts_accepts_standard_deal() {
    let dealer = 0;
    let mut hands = [Hand::empty(), Hand::empty(), Hand::empty(), Hand::empty()];

    for (seat, hand) in hands.iter_mut().enumerate() {
        let count = if seat == dealer {
            DEALER_HAND_SIZE
        } else {
            NON_DEALER_HAND_SIZE
        };
        for _ in 0..count {
            hand.concealed_mut().push(Tile::man(1));
        }
    }

    validate_deal_counts(&hands, dealer).unwrap();
}

#[test]
fn validate_deal_counts_rejects_wrong_size() {
    let mut hands = [Hand::empty(), Hand::empty(), Hand::empty(), Hand::empty()];
    for _ in 0..13 {
        hands[0].concealed_mut().push(Tile::man(1));
    }

    assert!(validate_deal_counts(&hands, 0).is_err());
}
