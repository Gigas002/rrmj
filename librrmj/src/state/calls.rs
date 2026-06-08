use crate::action::Action;
use crate::hand::{Concealed, Meld};
use crate::tile::{Tile, TileIdentity};
use crate::Error;

use super::next_seat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallKind {
    Chi,
    Pon,
    OpenKan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCall {
    pub kind: CallKind,
    pub meld: Meld,
    pub remove_from_concealed: Vec<Tile>,
}

pub fn kamicha(discarder: usize) -> usize {
    next_seat(discarder)
}

pub fn chi_actions(concealed: &Concealed, called: Tile) -> Vec<Action> {
    let Some(suit) = called.suit() else {
        return Vec::new();
    };
    let Some(rank) = called.rank() else {
        return Vec::new();
    };

    let mut actions = Vec::new();
    for &[low, mid, high] in &[
        [rank - 2, rank - 1, rank],
        [rank - 1, rank, rank + 1],
        [rank, rank + 1, rank + 2],
    ] {
        if low < 1 || high > 9 {
            continue;
        }
        let sequence = [
            Tile::numbered(suit, low),
            Tile::numbered(suit, mid),
            Tile::numbered(suit, high),
        ];
        if let Ok(tiles) = pick_chi_tiles(concealed, called, sequence) {
            actions.push(Action::Chi { tiles });
        }
    }

    actions
}

pub fn can_pon(concealed: &Concealed, called: Tile) -> bool {
    matching_tiles(concealed, called).len() >= 2
}

pub fn can_open_kan(concealed: &Concealed, called: Tile) -> bool {
    matching_tiles(concealed, called).len() >= 3
}

pub fn closed_kan_options(concealed: &Concealed) -> Vec<Tile> {
    let mut counts: Vec<(TileIdentity, Tile, usize)> = Vec::new();

    for &tile in concealed.tiles() {
        let identity = tile.identity();
        if let Some(entry) = counts.iter_mut().find(|(id, _, _)| *id == identity) {
            entry.2 += 1;
        } else {
            counts.push((identity, tile, 1));
        }
    }

    counts
        .into_iter()
        .filter(|(_, _, count)| *count >= 4)
        .map(|(_, tile, _)| tile)
        .collect()
}

pub fn resolve_chi(
    concealed: &Concealed,
    called: Tile,
    tiles: [Tile; 3],
) -> Result<ResolvedCall, Error> {
    if !is_valid_chi_sequence(tiles) {
        return Err(Error::InvalidChiSequence { tiles });
    }
    if !tiles.iter().any(|t| t.matches_identity(called)) {
        return Err(Error::InvalidCall {
            kind: CallKind::Chi,
            reason: "called tile not in meld",
        });
    }

    let mut remove_from_concealed = Vec::with_capacity(2);
    let mut remaining = concealed.tiles().to_vec();
    for tile in tiles {
        if tile.matches_identity(called) {
            continue;
        }
        let pos = remaining
            .iter()
            .position(|t| *t == tile)
            .ok_or(Error::TileNotInHand { tile })?;
        remove_from_concealed.push(remaining.remove(pos));
    }

    if remove_from_concealed.len() != 2 {
        return Err(Error::InvalidCall {
            kind: CallKind::Chi,
            reason: "concealed tiles do not match chii",
        });
    }

    let meld = Meld::chi(tiles, called)?;
    Ok(ResolvedCall {
        kind: CallKind::Chi,
        meld,
        remove_from_concealed,
    })
}

pub fn resolve_pon(concealed: &Concealed, called: Tile) -> Result<ResolvedCall, Error> {
    let remove_from_concealed = matching_tiles(concealed, called);
    if remove_from_concealed.len() < 2 {
        return Err(Error::InvalidCall {
            kind: CallKind::Pon,
            reason: "need two matching concealed tiles",
        });
    }
    let remove_from_concealed = remove_from_concealed[..2].to_vec();

    let tiles = [
        remove_from_concealed[0],
        remove_from_concealed[1],
        called,
    ];
    let meld = Meld::pon(tiles, called)?;
    Ok(ResolvedCall {
        kind: CallKind::Pon,
        meld,
        remove_from_concealed,
    })
}

pub fn resolve_open_kan(concealed: &Concealed, called: Tile) -> Result<ResolvedCall, Error> {
    let remove_from_concealed = matching_tiles(concealed, called);
    if remove_from_concealed.len() < 3 {
        return Err(Error::InvalidCall {
            kind: CallKind::OpenKan,
            reason: "need three matching concealed tiles",
        });
    }
    let remove_from_concealed = remove_from_concealed[..3].to_vec();

    let tiles = [
        remove_from_concealed[0],
        remove_from_concealed[1],
        remove_from_concealed[2],
        called,
    ];
    let meld = Meld::open_kan(tiles, called)?;
    Ok(ResolvedCall {
        kind: CallKind::OpenKan,
        meld,
        remove_from_concealed,
    })
}

pub fn resolve_closed_kan(concealed: &Concealed, tile: Tile) -> Result<Meld, Error> {
    let matching = matching_tiles(concealed, tile);
    if matching.len() < 4 {
        return Err(Error::InvalidCall {
            kind: CallKind::OpenKan,
            reason: "need four matching concealed tiles for closed kan",
        });
    }

    Meld::closed_kan([
        matching[0],
        matching[1],
        matching[2],
        matching[3],
    ])
}

pub fn call_priority(kind: CallKind) -> u8 {
    match kind {
        CallKind::OpenKan | CallKind::Pon => 2,
        CallKind::Chi => 1,
    }
}

pub fn seat_priority(discarder: usize, seat: usize) -> u8 {
    ((seat + 4 - discarder) % 4) as u8
}

fn matching_tiles(concealed: &Concealed, called: Tile) -> Vec<Tile> {
    concealed
        .tiles()
        .iter()
        .copied()
        .filter(|t| t.matches_identity(called))
        .collect()
}

fn pick_chi_tiles(
    concealed: &Concealed,
    called: Tile,
    sequence: [Tile; 3],
) -> Result<[Tile; 3], ()> {
    let mut remaining = concealed.tiles().to_vec();
    let mut picked = [Tile::man(1); 3];
    let mut pick_index = 0;

    for tile in sequence {
        if tile.matches_identity(called) {
            picked[pick_index] = called;
            pick_index += 1;
            continue;
        }
        let pos = remaining.iter().position(|t| *t == tile).ok_or(())?;
        picked[pick_index] = remaining.remove(pos);
        pick_index += 1;
    }

    if pick_index == 3 {
        Ok(picked)
    } else {
        Err(())
    }
}

fn is_valid_chi_sequence(tiles: [Tile; 3]) -> bool {
    let Some(suit) = tiles[0].suit() else {
        return false;
    };
    if !tiles.iter().all(|t| t.suit() == Some(suit)) {
        return false;
    }

    let mut ranks: Vec<u8> = tiles.iter().filter_map(|t| t.rank()).collect();
    ranks.sort_unstable();
    ranks.windows(2).all(|w| w[1] == w[0] + 1)
}
