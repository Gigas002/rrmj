//! Pattern-yaku detection via winning-hand decomposition.

use crate::hand::{Hand, KanForm, MeldKind};
use crate::rules::profile::WinContext;
use crate::tile::{Suit, Tile};

use super::win::tiles_with_win_tile;

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

    pub(crate) fn tiles(self) -> Vec<Tile> {
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
                Some(MeldComponent::from_triplet(meld.tiles()[0].plain(), true))
            }
            MeldKind::Kan(KanForm::Closed) => {
                Some(MeldComponent::from_triplet(meld.tiles()[0].plain(), false))
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
