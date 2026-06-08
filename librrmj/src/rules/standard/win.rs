use crate::hand::{Hand, MeldKind};
use crate::tile::{Suit, Tile};

pub fn is_winning_hand(hand: &Hand, win_tile: Option<Tile>) -> bool {
    let mut concealed = hand.concealed().tiles().to_vec();
    if let Some(tile) = win_tile
        && (concealed.len() < 14 || !concealed.contains(&tile))
    {
        concealed.push(tile);
    }
    concealed.sort();

    let open_sets = hand.melds().len();
    if open_sets > 4 {
        return false;
    }

    if hand.melds().is_empty() && is_chiitoitsu(&concealed) {
        return true;
    }

    let sets_needed = 4usize.saturating_sub(open_sets);
    can_form_mentsu_and_pair(&concealed, sets_needed)
}

pub fn is_tenpai(hand: &Hand) -> bool {
    for tile in all_wait_tiles() {
        if is_winning_hand(hand, Some(tile)) {
            return true;
        }
    }
    false
}

fn is_chiitoitsu(tiles: &[Tile]) -> bool {
    if tiles.len() != 14 {
        return false;
    }
    tiles
        .chunks(2)
        .all(|pair| pair.len() == 2 && pair[0] == pair[1])
}

fn can_form_mentsu_and_pair(tiles: &[Tile], sets_needed: usize) -> bool {
    if sets_needed == 0 {
        return tiles.len() == 2 && tiles[0] == tiles[1];
    }

    for i in 0..tiles.len() {
        for j in (i + 1)..tiles.len() {
            if tiles[i] != tiles[j] {
                continue;
            }
            let mut rest = tiles.to_vec();
            rest.remove(j);
            rest.remove(i);
            if can_form_sets(&rest, sets_needed) {
                return true;
            }
        }
    }
    false
}

fn can_form_sets(tiles: &[Tile], sets_needed: usize) -> bool {
    if sets_needed == 0 {
        return tiles.is_empty();
    }
    if tiles.is_empty() {
        return false;
    }

    if tiles.len() >= 3
        && tiles[0] == tiles[1]
        && tiles[1] == tiles[2]
        && can_form_sets(&tiles[3..], sets_needed - 1)
    {
        return true;
    }

    if let (Some(suit), Some(rank)) = (tiles[0].suit(), tiles[0].rank())
        && rank <= 7
    {
        let t2 = Tile::numbered(suit, rank + 1);
        let t3 = Tile::numbered(suit, rank + 2);
        if let Some(rest) = remove_tiles(tiles, &[tiles[0], t2, t3])
            && can_form_sets(&rest, sets_needed - 1)
        {
            return true;
        }
    }

    false
}

fn remove_tiles(tiles: &[Tile], to_remove: &[Tile]) -> Option<Vec<Tile>> {
    let mut rest = tiles.to_vec();
    for tile in to_remove {
        let pos = rest.iter().position(|t| *t == *tile)?;
        rest.remove(pos);
    }
    Some(rest)
}

fn all_wait_tiles() -> Vec<Tile> {
    let mut tiles = Vec::new();
    for suit in Suit::ALL {
        for rank in 1..=9 {
            tiles.push(Tile::numbered(suit, rank));
            if rank == 5 {
                tiles.push(Tile::red_five(suit));
            }
        }
    }
    for wind in crate::tile::Wind::ALL {
        tiles.push(Tile::wind(wind));
    }
    for dragon in crate::tile::Dragon::ALL {
        tiles.push(Tile::dragon(dragon));
    }
    tiles
}

pub fn is_simple(tile: Tile) -> bool {
    matches!(tile.rank(), Some(2..=8))
}

pub fn hand_contains_yakuhai(hand: &Hand, tile: Tile) -> bool {
    let identity = tile.identity();
    hand.concealed()
        .tiles()
        .iter()
        .any(|t| t.identity() == identity)
        || hand.melds().iter().any(|meld| {
            meld.tiles()
                .iter()
                .any(|t| t.identity() == identity && meld.kind() != MeldKind::Chi)
        })
}

pub fn all_tiles_in_hand(hand: &Hand, win_tile: Tile) -> Vec<Tile> {
    let mut tiles = hand.concealed().tiles().to_vec();
    if !tiles.contains(&win_tile) {
        tiles.push(win_tile);
    }
    for meld in hand.melds() {
        tiles.extend_from_slice(meld.tiles());
    }
    tiles.sort();
    tiles
}

pub fn is_tanyao_hand(hand: &Hand, win_tile: Tile) -> bool {
    all_tiles_in_hand(hand, win_tile)
        .iter()
        .all(|tile| is_simple(*tile))
}

pub fn is_pinfu_hand(hand: &Hand, win_tile: Tile) -> bool {
    if !hand.melds().is_empty() {
        return false;
    }
    is_winning_hand(hand, Some(win_tile))
}
