use crate::action::{Action, KanIntent};
use crate::agent::PlayerView;
use crate::hand::{Hand, Meld, MeldKind};
use crate::tile::Tile;

use super::shanten::hand_from_parts;

pub fn prefer_win(legal: &[Action]) -> Option<Action> {
    legal
        .iter()
        .copied()
        .find(|action| matches!(action, Action::Tsumo | Action::Ron))
}

pub fn hand_from_view(view: &PlayerView) -> Option<Hand> {
    super::shanten::hand_from_parts(
        view.own_concealed.clone(),
        view.seats[view.seat].melds.clone(),
    )
}

pub fn simulate_call(hand: &Hand, action: Action, called: Tile) -> Option<Hand> {
    let mut melds = hand.melds().to_vec();
    let mut concealed = hand.concealed().tiles().to_vec();

    match action {
        Action::Pon => {
            remove_matching_identity(&mut concealed, called, 2)?;
            melds.push(Meld::pon([called, called, called], called).ok()?);
        }
        Action::Kan(KanIntent::Open) => {
            remove_matching_identity(&mut concealed, called, 3)?;
            let tile = called;
            melds.push(Meld::open_kan([tile, tile, tile, tile], called).ok()?);
        }
        Action::Chi { tiles } => {
            for tile in tiles {
                if tile == called {
                    continue;
                }
                let pos = concealed.iter().position(|t| *t == tile)?;
                concealed.remove(pos);
            }
            let mut chi_tiles = tiles;
            chi_tiles.sort();
            melds.push(Meld::chi(chi_tiles, called).ok()?);
        }
        _ => return None,
    }

    hand_from_parts(concealed, melds)
}

pub fn simulate_closed_kan(hand: &Hand, tile: Tile) -> Option<Hand> {
    let mut concealed = hand.concealed().tiles().to_vec();
    remove_matching_identity(&mut concealed, tile, 4)?;
    let mut melds = hand.melds().to_vec();
    melds.push(Meld::closed_kan([tile, tile, tile, tile]).ok()?);
    hand_from_parts(concealed, melds)
}

pub fn simulate_added_kan(hand: &Hand, meld_index: usize) -> Option<Hand> {
    let meld = hand.melds().get(meld_index)?;
    if meld.kind() != MeldKind::Pon {
        return None;
    }
    let identity = meld.tiles()[0].identity();
    let mut concealed = hand.concealed().tiles().to_vec();
    let pos = concealed.iter().position(|t| t.identity() == identity)?;
    concealed.remove(pos);
    let called = meld.called()?;
    let tile = meld.tiles()[0];
    let mut melds = hand.melds().to_vec();
    melds[meld_index] = Meld::open_kan([tile, tile, tile, tile], called).ok()?;
    hand_from_parts(concealed, melds)
}

fn remove_matching_identity(concealed: &mut Vec<Tile>, tile: Tile, count: usize) -> Option<()> {
    for _ in 0..count {
        let pos = concealed.iter().position(|t| t.matches_identity(tile))?;
        concealed.remove(pos);
    }
    Some(())
}
