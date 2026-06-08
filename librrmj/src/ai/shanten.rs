use crate::hand::Hand;
use crate::rules::standard::is_winning_hand;
use crate::tile::{Dragon, Suit, Tile, Wind};

/// Number of distinct tiles that complete the hand (ukeire). Higher is better.
pub fn waiting_count(hand: &Hand) -> usize {
    let concealed_len = hand.concealed().len();
    if concealed_len % 3 == 2 {
        return usize::from(is_winning_hand(hand, None));
    }
    if concealed_len % 3 != 1 {
        return 0;
    }

    candidate_tiles()
        .into_iter()
        .filter(|tile| is_winning_hand(hand, Some(*tile)))
        .count()
}

/// Best ukeire after discarding down to a 13-, 10-, … tile concealed shape.
pub fn best_waiting_potential(hand: &Hand) -> usize {
    let len = hand.concealed().len();
    if len % 3 == 1 {
        return waiting_count(hand);
    }
    if len == 14 && is_winning_hand(hand, None) {
        return candidate_tiles().len();
    }

    let mut best = 0usize;
    let tiles: Vec<Tile> = hand.concealed().tiles().to_vec();
    let mut seen = Vec::new();
    for tile in tiles {
        if seen.contains(&tile) {
            continue;
        }
        seen.push(tile);
        if let Some(after) = hand_without_concealed_tile(hand, tile) {
            best = best.max(best_waiting_potential(&after));
        }
    }
    best
}

pub(crate) fn hand_from_parts(concealed: Vec<Tile>, melds: Vec<crate::hand::Meld>) -> Option<Hand> {
    Hand::new(crate::hand::Concealed::from_tiles(concealed), melds).ok()
}

pub(crate) fn hand_without_concealed_tile(hand: &Hand, tile: Tile) -> Option<Hand> {
    let mut concealed = hand.concealed().tiles().to_vec();
    let pos = concealed.iter().position(|t| *t == tile)?;
    concealed.remove(pos);
    hand_from_parts(concealed, hand.melds().to_vec())
}

fn candidate_tiles() -> Vec<Tile> {
    let mut tiles = Vec::new();
    for suit in Suit::ALL {
        for rank in 1..=9 {
            tiles.push(Tile::numbered(suit, rank));
            if rank == 5 {
                tiles.push(Tile::red_five(suit));
            }
        }
    }
    for wind in Wind::ALL {
        tiles.push(Tile::wind(wind));
    }
    for dragon in Dragon::ALL {
        tiles.push(Tile::dragon(dragon));
    }
    tiles
}
