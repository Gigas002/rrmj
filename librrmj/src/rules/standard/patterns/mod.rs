//! Pattern-yaku detection via winning-hand decomposition.

#[cfg(test)]
mod tests;

use crate::hand::{Hand, KanForm, MeldKind};
use crate::rules::profile::WinContext;
use crate::tile::{Suit, Tile, TileKind};

use super::win::{
    all_tiles_in_hand, is_chiitoitsu_hand, is_terminal_or_honor, tiles_with_win_tile,
};

/// One meld in a standard-form decomposition (four melds + pair).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MeldComponent {
    pub sequence: Option<(Suit, u8)>,
    pub triplet: Option<Tile>,
    pub open: bool,
}

impl MeldComponent {
    fn from_triplet(tile: Tile, open: bool) -> Self {
        Self {
            sequence: None,
            triplet: Some(tile),
            open,
        }
    }

    fn from_sequence(suit: Suit, low_rank: u8, open: bool) -> Self {
        Self {
            sequence: Some((suit, low_rank)),
            triplet: None,
            open,
        }
    }

    fn is_triplet(self) -> bool {
        self.triplet.is_some()
    }

    fn tiles(self) -> Vec<Tile> {
        match (self.sequence, self.triplet) {
            (Some((suit, rank)), None) => [
                Tile::numbered(suit, rank),
                Tile::numbered(suit, rank + 1),
                Tile::numbered(suit, rank + 2),
            ]
            .to_vec(),
            (None, Some(tile)) => vec![tile, tile, tile],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Decomposition {
    pub(crate) melds: [MeldComponent; 4],
    pub(crate) pair: Tile,
}

pub fn is_toitoi_hand(ctx: &WinContext<'_>) -> bool {
    if is_chiitoitsu_hand(ctx.hand(), ctx.win_tile, ctx.win_type) {
        return false;
    }
    decompositions(ctx)
        .into_iter()
        .any(|d| d.melds.iter().all(|m| m.is_triplet()))
}

pub fn is_sanshoku_hand(ctx: &WinContext<'_>) -> bool {
    decompositions(ctx)
        .into_iter()
        .any(|d| has_sanshoku(&d.melds))
}

pub fn is_ittsu_hand(ctx: &WinContext<'_>) -> bool {
    decompositions(ctx).into_iter().any(|d| has_ittsu(&d.melds))
}

pub fn is_iipeikou_hand(ctx: &WinContext<'_>) -> bool {
    if !ctx.is_menzen() {
        return false;
    }
    decompositions(ctx)
        .into_iter()
        .any(|d| peikou_pair_count(&d.melds) == 1)
}

pub fn is_ryanpeikou_hand(ctx: &WinContext<'_>) -> bool {
    if !ctx.is_menzen() {
        return false;
    }
    decompositions(ctx)
        .into_iter()
        .any(|d| peikou_pair_count(&d.melds) == 2)
}

pub fn is_honitsu_hand(ctx: &WinContext<'_>) -> bool {
    if is_chinitsu_hand(ctx) {
        return false;
    }
    let tiles = all_tiles_in_hand(ctx.hand(), ctx.win_tile, ctx.win_type);
    flush_suit(&tiles).is_some() && tiles.iter().any(|t| is_honor(*t))
}

pub fn is_chinitsu_hand(ctx: &WinContext<'_>) -> bool {
    let tiles = all_tiles_in_hand(ctx.hand(), ctx.win_tile, ctx.win_type);
    flush_suit(&tiles).is_some() && tiles.iter().all(|t| t.suit().is_some())
}

pub fn is_chanta_hand(ctx: &WinContext<'_>) -> bool {
    if is_junchan_hand(ctx) {
        return false;
    }
    if is_chiitoitsu_hand(ctx.hand(), ctx.win_tile, ctx.win_type) {
        return chiitoitsu_pairs_all(ctx, is_terminal_or_honor);
    }
    decompositions(ctx).into_iter().any(|d| {
        d.melds.iter().all(|m| meld_has_terminal_or_honor(*m)) && is_terminal_or_honor(d.pair)
    })
}

pub fn is_junchan_hand(ctx: &WinContext<'_>) -> bool {
    if is_chiitoitsu_hand(ctx.hand(), ctx.win_tile, ctx.win_type) {
        return chiitoitsu_pairs_all(ctx, is_numbered_terminal);
    }
    let tiles = all_tiles_in_hand(ctx.hand(), ctx.win_tile, ctx.win_type);
    if tiles.iter().any(|t| is_honor(*t)) {
        return false;
    }
    decompositions(ctx).into_iter().any(|d| {
        d.melds.iter().all(|m| meld_has_numbered_terminal(*m)) && is_numbered_terminal(d.pair)
    })
}

fn chiitoitsu_pairs_all(ctx: &WinContext<'_>, pred: impl Fn(Tile) -> bool) -> bool {
    let mut tiles = tiles_with_win_tile(ctx.hand(), ctx.win_tile, ctx.win_type);
    tiles.sort();
    tiles
        .chunks(2)
        .all(|pair| pair.len() == 2 && pair[0] == pair[1] && pred(pair[0]))
}

fn is_honor(tile: Tile) -> bool {
    matches!(tile.kind(), TileKind::Wind(_) | TileKind::Dragon(_))
}

fn is_numbered_terminal(tile: Tile) -> bool {
    matches!(tile.rank(), Some(1 | 9))
}

fn flush_suit(tiles: &[Tile]) -> Option<Suit> {
    let mut suit = None;
    for tile in tiles {
        let Some(s) = tile.suit() else {
            continue;
        };
        match suit {
            None => suit = Some(s),
            Some(existing) if existing == s => {}
            _ => return None,
        }
    }
    suit
}

fn has_sanshoku(melds: &[MeldComponent; 4]) -> bool {
    for rank in 1..=7u8 {
        let mut suits = [false; 3];
        for meld in melds {
            if let Some((suit, low)) = meld.sequence
                && low == rank
            {
                suits[suit_index(suit)] = true;
            }
        }
        if suits == [true, true, true] {
            return true;
        }
    }
    false
}

fn has_ittsu(melds: &[MeldComponent; 4]) -> bool {
    for suit in Suit::ALL {
        let mut rows = [false; 3];
        for meld in melds {
            if let Some((s, low)) = meld.sequence
                && s == suit
            {
                match low {
                    1 => rows[0] = true,
                    4 => rows[1] = true,
                    7 => rows[2] = true,
                    _ => {}
                }
            }
        }
        if rows == [true, true, true] {
            return true;
        }
    }
    false
}

fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Man => 0,
        Suit::Pin => 1,
        Suit::Sou => 2,
    }
}

fn peikou_pair_count(melds: &[MeldComponent; 4]) -> usize {
    let mut concealed_sequences: Vec<(Suit, u8)> = melds
        .iter()
        .filter(|m| !m.open)
        .filter_map(|m| m.sequence)
        .collect();
    concealed_sequences.sort_by_key(|(suit, rank)| (suit_index(*suit), *rank));
    let mut pairs = 0usize;
    let mut i = 0;
    while i + 1 < concealed_sequences.len() {
        if concealed_sequences[i] == concealed_sequences[i + 1] {
            pairs += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    pairs
}

fn meld_has_terminal_or_honor(meld: MeldComponent) -> bool {
    meld.tiles().iter().any(|&t| is_terminal_or_honor(t))
}

fn meld_has_numbered_terminal(meld: MeldComponent) -> bool {
    meld.tiles().iter().any(|&t| is_numbered_terminal(t))
}

pub(crate) fn decompositions(ctx: &WinContext<'_>) -> Vec<Decomposition> {
    let hand = ctx.hand();
    let open = open_melds(hand);
    if open.len() > 4 {
        return vec![];
    }
    let sets_needed = 4usize.saturating_sub(open.len());
    let mut tiles = tiles_with_win_tile(hand, ctx.win_tile, ctx.win_type);
    tiles.sort();

    let mut out = Vec::new();
    for i in 0..tiles.len() {
        for j in (i + 1)..tiles.len() {
            if tiles[i] != tiles[j] {
                continue;
            }
            let pair = tiles[i];
            let mut rest = tiles.clone();
            rest.remove(j);
            rest.remove(i);
            let mut concealed = Vec::new();
            enumerate_concealed_sets(&rest, sets_needed, &mut concealed, &mut |sets| {
                let mut melds = open.clone();
                melds.extend_from_slice(sets);
                if melds.len() == 4 {
                    let array: [MeldComponent; 4] = melds.try_into().expect("four melds");
                    out.push(Decomposition { melds: array, pair });
                }
            });
        }
    }
    out
}

fn open_melds(hand: &Hand) -> Vec<MeldComponent> {
    hand.melds()
        .iter()
        .filter_map(|meld| match meld.kind() {
            MeldKind::Chi => {
                let mut tiles = meld.tiles().to_vec();
                tiles.sort_by(|a, b| a.cmp_sort(*b));
                let suit = tiles[0].suit()?;
                let rank = tiles[0].rank()?;
                Some(MeldComponent::from_sequence(suit, rank, true))
            }
            MeldKind::Pon | MeldKind::Kan(KanForm::Open) => {
                Some(MeldComponent::from_triplet(meld.tiles()[0], true))
            }
            MeldKind::Kan(KanForm::Closed) => {
                Some(MeldComponent::from_triplet(meld.tiles()[0], false))
            }
        })
        .collect()
}

fn enumerate_concealed_sets(
    tiles: &[Tile],
    sets_needed: usize,
    current: &mut Vec<MeldComponent>,
    emit: &mut dyn FnMut(&[MeldComponent]),
) {
    if sets_needed == 0 {
        if tiles.is_empty() {
            emit(current);
        }
        return;
    }
    if tiles.len() < 3 {
        return;
    }

    if tiles[0] == tiles[1] && tiles[1] == tiles[2] {
        current.push(MeldComponent::from_triplet(tiles[0], false));
        enumerate_concealed_sets(&tiles[3..], sets_needed - 1, current, emit);
        current.pop();
    }

    if let (Some(suit), Some(rank)) = (tiles[0].suit(), tiles[0].rank())
        && rank <= 7
    {
        let t2 = Tile::numbered(suit, rank + 1);
        let t3 = Tile::numbered(suit, rank + 2);
        if let Some(rest) = remove_tiles(tiles, &[tiles[0], t2, t3]) {
            current.push(MeldComponent::from_sequence(suit, rank, false));
            enumerate_concealed_sets(&rest, sets_needed - 1, current, emit);
            current.pop();
        }
    }
}

fn remove_tiles(tiles: &[Tile], to_remove: &[Tile]) -> Option<Vec<Tile>> {
    let mut rest = tiles.to_vec();
    for tile in to_remove {
        let pos = rest.iter().position(|t| *t == *tile)?;
        rest.remove(pos);
    }
    Some(rest)
}
