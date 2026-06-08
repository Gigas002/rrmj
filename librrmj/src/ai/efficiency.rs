use crate::agent::PlayerView;
use crate::hand::Hand;
use crate::rules::standard::is_winning_hand;
use crate::tile::{Dragon, Suit, Tile, TileIdentity, Wind};

/// Ukeire weighted by how many copies of each wait remain unseen.
pub fn weighted_waiting_count(hand: &Hand, view: &PlayerView) -> u32 {
    let concealed_len = hand.concealed().len();
    if concealed_len % 3 == 2 {
        return u32::from(is_winning_hand(hand, None)) * 4;
    }
    if concealed_len % 3 != 1 {
        return 0;
    }

    candidate_tiles()
        .into_iter()
        .filter(|tile| is_winning_hand(hand, Some(*tile)))
        .map(|tile| u32::from(remaining_copies(view, tile)))
        .sum()
}

/// Copies of `tile` not yet visible on the table or in the concealed hand.
pub fn remaining_copies(view: &PlayerView, tile: Tile) -> u8 {
    4u8.saturating_sub(visible_identity_count(view, tile.identity()))
}

pub fn visible_identity_count(view: &PlayerView, identity: TileIdentity) -> u8 {
    let mut count = 0u8;
    for tile in view.own_concealed.iter().copied() {
        if tile.identity() == identity {
            count = count.saturating_add(1);
        }
    }
    for seat in &view.seats {
        for tile in &seat.discards {
            if tile.identity() == identity {
                count = count.saturating_add(1);
            }
        }
        for meld in &seat.melds {
            for tile in meld.tiles() {
                if tile.identity() == identity {
                    count = count.saturating_add(1);
                }
            }
        }
    }
    count
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
