use crate::agent::PlayerView;
use crate::hand::Hand;
use crate::rules::standard::is_winning_hand;
use crate::tile::{Dragon, Suit, Tile, TileKind, Wind};

use super::efficiency::{remaining_copies, weighted_waiting_count};

/// Shanten: `-1` = complete hand, `0` = tenpai, `n` = n tiles from tenpai.
///
/// Works on 13- and 14-tile shapes (counting open melds as sets): a 14-tile
/// hand reports the shanten left after its best discard.
pub fn shanten_to_tenpai(hand: &Hand) -> i8 {
    let counts = kind_counts(hand.concealed().tiles());
    let melds = hand.melds().len() as i8;
    let mut best = normal_shanten(&counts, melds);
    if melds == 0 {
        best = best
            .min(chiitoitsu_shanten(&counts))
            .min(kokushi_shanten(&counts));
    }
    best
}

/// Tile kinds 0..34: man 1–9, pin 1–9, sou 1–9, winds E S W N, dragons W G R.
pub(crate) fn kind_index(tile: Tile) -> usize {
    match tile.kind() {
        TileKind::Man(rank) => usize::from(rank - 1),
        TileKind::Pin(rank) => 9 + usize::from(rank - 1),
        TileKind::Sou(rank) => 18 + usize::from(rank - 1),
        TileKind::Wind(wind) => 27 + Wind::ALL.iter().position(|w| *w == wind).unwrap_or(0),
        TileKind::Dragon(dragon) => 31 + Dragon::ALL.iter().position(|d| *d == dragon).unwrap_or(0),
    }
}

fn kind_counts(tiles: &[Tile]) -> [u8; 34] {
    let mut counts = [0u8; 34];
    for &tile in tiles {
        counts[kind_index(tile)] += 1;
    }
    counts
}

fn normal_shanten(counts: &[u8; 34], open_melds: i8) -> i8 {
    let mut work = *counts;
    let mut best = search_sets(&mut work, 0, open_melds, 0);
    for kind in 0..34 {
        if work[kind] >= 2 {
            work[kind] -= 2;
            best = best.min(search_sets(&mut work, 0, open_melds, 0) - 1);
            work[kind] += 2;
        }
    }
    best
}

/// Depth-first split into sets and partial sets (pairs, two-sided, edge, gap).
fn search_sets(counts: &mut [u8; 34], start: usize, sets: i8, partials: i8) -> i8 {
    let mut kind = start;
    while kind < 34 && counts[kind] == 0 {
        kind += 1;
    }
    if kind == 34 {
        let partials = partials.min(4 - sets);
        return 8 - 2 * sets - partials;
    }

    let suited = kind < 27;
    let rank = kind % 9;
    let mut best = i8::MAX;

    if counts[kind] >= 3 {
        counts[kind] -= 3;
        best = best.min(search_sets(counts, kind, sets + 1, partials));
        counts[kind] += 3;
    }
    if suited && rank <= 6 && counts[kind + 1] > 0 && counts[kind + 2] > 0 {
        counts[kind] -= 1;
        counts[kind + 1] -= 1;
        counts[kind + 2] -= 1;
        best = best.min(search_sets(counts, kind, sets + 1, partials));
        counts[kind] += 1;
        counts[kind + 1] += 1;
        counts[kind + 2] += 1;
    }
    if sets + partials < 4 {
        if counts[kind] >= 2 {
            counts[kind] -= 2;
            best = best.min(search_sets(counts, kind, sets, partials + 1));
            counts[kind] += 2;
        }
        if suited && rank <= 7 && counts[kind + 1] > 0 {
            counts[kind] -= 1;
            counts[kind + 1] -= 1;
            best = best.min(search_sets(counts, kind, sets, partials + 1));
            counts[kind] += 1;
            counts[kind + 1] += 1;
        }
        if suited && rank <= 6 && counts[kind + 2] > 0 {
            counts[kind] -= 1;
            counts[kind + 2] -= 1;
            best = best.min(search_sets(counts, kind, sets, partials + 1));
            counts[kind] += 1;
            counts[kind + 2] += 1;
        }
    }
    // Leave one copy of this kind unused.
    counts[kind] -= 1;
    best = best.min(search_sets(counts, kind, sets, partials));
    counts[kind] += 1;
    best
}

fn chiitoitsu_shanten(counts: &[u8; 34]) -> i8 {
    let pairs = counts.iter().filter(|&&c| c >= 2).count() as i8;
    let kinds = counts.iter().filter(|&&c| c >= 1).count() as i8;
    6 - pairs + (7 - kinds).max(0)
}

fn kokushi_shanten(counts: &[u8; 34]) -> i8 {
    const TERMINALS: [usize; 13] = [0, 8, 9, 17, 18, 26, 27, 28, 29, 30, 31, 32, 33];
    let kinds = TERMINALS.iter().filter(|&&k| counts[k] >= 1).count() as i8;
    let pair = TERMINALS.iter().any(|&k| counts[k] >= 2);
    13 - kinds - i8::from(pair)
}

/// Tile kinds (one representative tile each, no red fives) that lower the
/// shanten of a 13-tile shape.
pub(crate) fn improving_tiles(hand: &Hand) -> Vec<Tile> {
    let base = shanten_to_tenpai(hand);
    let mut concealed = hand.concealed().tiles().to_vec();
    let mut out = Vec::new();
    for tile in candidate_tiles() {
        concealed.push(tile);
        if let Some(with) = hand_from_parts(concealed.clone(), hand.melds().to_vec())
            && shanten_to_tenpai(&with) < base
        {
            out.push(tile);
        }
        concealed.pop();
    }
    out
}

/// Aggregate hand quality for AI comparisons (lower shanten is better).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandStrength {
    pub shanten: i8,
    pub ukeire: usize,
    pub weighted: u32,
}

impl HandStrength {
    pub fn improves_over(self, baseline: Self, accept_equal_ukeire: bool) -> bool {
        if self.shanten != baseline.shanten {
            return self.shanten < baseline.shanten;
        }
        if self.ukeire != baseline.ukeire {
            return self.ukeire > baseline.ukeire
                || (accept_equal_ukeire && self.ukeire == baseline.ukeire);
        }
        self.weighted > baseline.weighted
    }

    pub fn is_better_than(self, other: Self) -> bool {
        self.shanten < other.shanten
            || (self.shanten == other.shanten && self.ukeire > other.ukeire)
            || (self.shanten == other.shanten
                && self.ukeire == other.ukeire
                && self.weighted > other.weighted)
    }
}

pub fn evaluate_hand(hand: &Hand, view: Option<&PlayerView>) -> HandStrength {
    HandStrength {
        shanten: shanten_to_tenpai(hand),
        ukeire: best_waiting_potential(hand),
        weighted: view.map_or(0, |v| best_weighted_potential(hand, v)),
    }
}

fn best_weighted_potential(hand: &Hand, view: &PlayerView) -> u32 {
    let len = hand.concealed().len();
    if len % 3 == 1 {
        return improving_tiles(hand)
            .into_iter()
            .map(|tile| u32::from(remaining_copies(view, tile)))
            .sum();
    }
    if len % 3 == 2 && is_winning_hand(hand, None) {
        return weighted_waiting_count(hand, view).max(4);
    }
    if len % 3 == 2 {
        // Ukeire only counts among discards that keep the best shanten.
        let target = shanten_to_tenpai(hand);
        let mut best = 0u32;
        let mut seen = Vec::new();
        for tile in hand.concealed().tiles() {
            if seen.contains(&tile) {
                continue;
            }
            seen.push(tile);
            if let Some(after) = hand_without_concealed_tile(hand, *tile)
                && shanten_to_tenpai(&after) == target
            {
                best = best.max(best_weighted_potential(&after, view));
            }
        }
        return best;
    }
    0
}

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

/// Ukeire: tile kinds that lower shanten (after the best discard for 14 tiles).
pub fn best_waiting_potential(hand: &Hand) -> usize {
    let len = hand.concealed().len();
    if len % 3 == 1 {
        return improving_tiles(hand).len();
    }
    if len % 3 == 2 && is_winning_hand(hand, None) {
        return candidate_tiles().len();
    }
    if len % 3 == 2 {
        let target = shanten_to_tenpai(hand);
        let mut best = 0usize;
        let mut seen = Vec::new();
        for tile in hand.concealed().tiles() {
            if seen.contains(&tile) {
                continue;
            }
            seen.push(tile);
            if let Some(after) = hand_without_concealed_tile(hand, *tile)
                && shanten_to_tenpai(&after) == target
            {
                best = best.max(best_waiting_potential(&after));
            }
        }
        return best;
    }

    0
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hand::Concealed;

    #[test]
    fn tenpai_hand_has_zero_shanten() {
        let concealed = vec![
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
        let hand = Hand::new(Concealed::from_tiles(concealed), vec![]).unwrap();
        assert_eq!(shanten_to_tenpai(&hand), 0);
    }

    #[test]
    fn best_waiting_potential_handles_eleven_tile_hand() {
        let concealed = vec![
            Tile::pin(2),
            Tile::pin(3),
            Tile::pin(4),
            Tile::sou(2),
            Tile::sou(3),
            Tile::sou(4),
            Tile::sou(5),
            Tile::sou(6),
            Tile::sou(7),
            Tile::sou(8),
            Tile::sou(9),
        ];
        let meld = crate::hand::Meld::pon([Tile::man(5), Tile::man(5), Tile::man(5)], Tile::man(5))
            .unwrap();
        let hand = Hand::new(Concealed::from_tiles(concealed), vec![meld]).unwrap();
        assert!(best_waiting_potential(&hand) > 0);
    }
}
