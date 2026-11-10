#[cfg(test)]
mod tests;

use crate::hand::{Concealed, Hand};
use crate::tile::{Suit, Tile, TileKind};

pub fn is_winning_hand(hand: &Hand, win_tile: Option<Tile>) -> bool {
    let held = hand.concealed().tiles();
    let mut concealed: Vec<Tile> = held.iter().map(|t| t.plain()).collect();
    if let Some(tile) = win_tile {
        // Ron waits have concealed.len() % 3 == 1 and need the called tile added.
        // Tsumo completes with concealed.len() % 3 == 2 and the draw already in hand
        // (matched exactly, so a red five is not "held" through a plain five).
        let ron_shape = concealed.len() % 3 == 1;
        if ron_shape || !held.contains(&tile) {
            concealed.push(tile.plain());
        }
    }
    concealed.sort();

    let open_sets = hand.melds().len();
    if open_sets > 4 {
        return false;
    }

    if hand.melds().is_empty() && is_chiitoitsu(&concealed) {
        return true;
    }

    if hand.melds().is_empty() && super::yaku::is_kokushi_tiles(&concealed) {
        return true;
    }

    if hand.melds().is_empty() && is_chuuren_tiles(&concealed) {
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

/// Tile kinds that complete a 13-tile hand (red fives count as fives).
pub fn wait_kinds(hand: &Hand) -> Vec<TileKind> {
    let mut kinds: Vec<TileKind> = all_wait_tiles()
        .into_iter()
        .filter(|tile| !tile.is_red() && is_winning_hand(hand, Some(*tile)))
        .map(|tile| tile.kind())
        .collect();
    kinds.dedup();
    kinds
}

/// During riichi, a closed kan is allowed only of the drawn tile and only
/// when the waits stay exactly the same.
pub fn riichi_kan_keeps_wait(hand: &Hand, drawn: Tile, kan_tile: Tile) -> bool {
    if drawn.kind() != kan_tile.kind() {
        return false;
    }
    let mut before = hand.concealed().tiles().to_vec();
    let Some(pos) = before.iter().position(|&t| t == drawn) else {
        return false;
    };
    before.remove(pos);
    let Ok(before) = Hand::new(Concealed::from_tiles(before), hand.melds().to_vec()) else {
        return false;
    };

    let (kan, rest): (Vec<Tile>, Vec<Tile>) = hand
        .concealed()
        .tiles()
        .iter()
        .partition(|t| t.kind() == kan_tile.kind());
    let Ok(kan) = <[Tile; 4]>::try_from(kan) else {
        return false;
    };
    let Ok(meld) = crate::hand::Meld::closed_kan(kan) else {
        return false;
    };
    let mut melds = hand.melds().to_vec();
    melds.push(meld);
    let Ok(after) = Hand::new(Concealed::from_tiles(rest), melds) else {
        return false;
    };

    let waits = wait_kinds(&before);
    !waits.is_empty() && waits == wait_kinds(&after)
}

/// Whether the hand stays tenpai after removing one tile from the concealed hand.
pub fn is_tenpai_after_discard(hand: &Hand, discard: Tile) -> bool {
    let mut concealed = hand.concealed().tiles().to_vec();
    let Some(pos) = concealed.iter().position(|&t| t == discard) else {
        return false;
    };
    concealed.remove(pos);
    let after = Hand::new(Concealed::from_tiles(concealed), hand.melds().to_vec())
        .expect("discard leaves valid hand shape");
    is_tenpai(&after)
}

pub fn is_chiitoitsu_hand(hand: &Hand, win_tile: Tile, win_type: crate::scoring::WinType) -> bool {
    if !hand.melds().is_empty() {
        return false;
    }
    let mut tiles = tiles_with_win_tile(hand, win_tile, win_type);
    tiles.sort();
    is_chiitoitsu(&tiles)
}

fn is_chuuren_tiles(tiles: &[Tile]) -> bool {
    crate::tile::Suit::ALL
        .iter()
        .any(|&suit| chuuren_suit_counts(tiles, suit).is_some())
}

fn chuuren_suit_counts(tiles: &[Tile], suit: crate::tile::Suit) -> Option<[u8; 9]> {
    let mut counts = [0u8; 9];
    for tile in tiles {
        let tile_suit = tile.suit()?;
        if tile_suit != suit {
            return None;
        }
        let rank = tile.rank()?;
        counts[(rank - 1) as usize] += 1;
    }
    if counts.iter().sum::<u8>() != 14 {
        return None;
    }
    if counts[0] >= 3 && counts[8] >= 3 && counts[1..8].iter().all(|&c| c >= 1) {
        Some(counts)
    } else {
        None
    }
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

pub(crate) fn all_wait_tiles() -> Vec<Tile> {
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

pub fn is_terminal_or_honor(tile: Tile) -> bool {
    match tile.kind() {
        TileKind::Man(r) | TileKind::Pin(r) | TileKind::Sou(r) => r == 1 || r == 9,
        TileKind::Wind(_) | TileKind::Dragon(_) => true,
    }
}

/// Every tile of the winning hand (melds included), red fives as plain fives.
pub fn all_tiles_in_hand(
    hand: &Hand,
    win_tile: Tile,
    win_type: crate::scoring::WinType,
) -> Vec<Tile> {
    let mut tiles: Vec<Tile> = all_tiles_with_reds(hand, win_tile, win_type)
        .into_iter()
        .map(Tile::plain)
        .collect();
    tiles.sort();
    tiles
}

/// Every tile of the winning hand as held, keeping the aka marker.
pub(crate) fn all_tiles_with_reds(
    hand: &Hand,
    win_tile: Tile,
    win_type: crate::scoring::WinType,
) -> Vec<Tile> {
    let mut tiles = raw_tiles_with_win_tile(hand, win_tile, win_type);
    for meld in hand.melds() {
        tiles.extend_from_slice(meld.tiles());
    }
    tiles
}

/// Concealed tiles plus the winning tile (ron always adds the called tile),
/// red fives as plain fives.
pub(crate) fn tiles_with_win_tile(
    hand: &Hand,
    win_tile: Tile,
    win_type: crate::scoring::WinType,
) -> Vec<Tile> {
    raw_tiles_with_win_tile(hand, win_tile, win_type)
        .into_iter()
        .map(Tile::plain)
        .collect()
}

fn raw_tiles_with_win_tile(
    hand: &Hand,
    win_tile: Tile,
    win_type: crate::scoring::WinType,
) -> Vec<Tile> {
    // A waiting hand (3n+1 tiles) needs the winning tile added; a tsumo hand
    // already holds the drawn tile. Same rule as `is_winning_hand`.
    let _ = win_type;
    let mut tiles = hand.concealed().tiles().to_vec();
    if tiles.len() % 3 == 1 {
        tiles.push(win_tile);
    }
    tiles
}
