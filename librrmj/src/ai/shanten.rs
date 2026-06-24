use crate::agent::PlayerView;
use crate::hand::Hand;
use crate::rules::standard::is_winning_hand;
use crate::tile::{Dragon, Suit, Tile, Wind};

use super::efficiency::weighted_waiting_count;

/// Minimum discards needed to reach tenpai. `-1` = winning shape, `0` = tenpai.
pub fn shanten_to_tenpai(hand: &Hand) -> i8 {
    let len = hand.concealed().len();
    if len % 3 == 2 && is_winning_hand(hand, None) {
        return -1;
    }
    if len % 3 == 1 && waiting_count(hand) > 0 {
        return 0;
    }
    match min_discards_to_tenpai(hand, 0, 8) {
        s if s > 8 => 8,
        s => s,
    }
}

fn min_discards_to_tenpai(hand: &Hand, depth: i8, limit: i8) -> i8 {
    if depth > limit {
        return limit + 1;
    }
    let len = hand.concealed().len();
    if len % 3 == 1 && waiting_count(hand) > 0 {
        return depth;
    }
    if len % 3 == 2 && is_winning_hand(hand, None) {
        return -1;
    }
    if len % 3 != 2 {
        return depth.saturating_add(1).min(limit + 1);
    }

    let mut best = limit + 1;
    let mut seen = Vec::new();
    for tile in hand.concealed().tiles() {
        if seen.contains(&tile) {
            continue;
        }
        seen.push(tile);
        if let Some(after) = hand_without_concealed_tile(hand, *tile) {
            best = best.min(min_discards_to_tenpai(&after, depth + 1, limit));
        }
    }
    best
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
        return weighted_waiting_count(hand, view);
    }
    if len % 3 == 2 && is_winning_hand(hand, None) {
        return weighted_waiting_count(hand, view).max(4);
    }
    if len % 3 == 2 {
        let mut best = 0u32;
        let mut seen = Vec::new();
        for tile in hand.concealed().tiles() {
            if seen.contains(&tile) {
                continue;
            }
            seen.push(tile);
            if let Some(after) = hand_without_concealed_tile(hand, *tile) {
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

/// Best ukeire after discarding down to a tenpai-check shape.
pub fn best_waiting_potential(hand: &Hand) -> usize {
    let len = hand.concealed().len();
    if len % 3 == 1 {
        return waiting_count(hand);
    }
    if len % 3 == 2 && is_winning_hand(hand, None) {
        return candidate_tiles().len();
    }
    if len % 3 == 2 {
        let mut best = 0usize;
        let mut seen = Vec::new();
        for tile in hand.concealed().tiles() {
            if seen.contains(&tile) {
                continue;
            }
            seen.push(tile);
            if let Some(after) = hand_without_concealed_tile(hand, *tile) {
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
        let meld = crate::hand::Meld::pon(
            [Tile::man(5), Tile::man(5), Tile::man(5)],
            Tile::man(5),
        )
        .unwrap();
        let hand = Hand::new(Concealed::from_tiles(concealed), vec![meld]).unwrap();
        assert!(best_waiting_potential(&hand) > 0);
    }
}
