use crate::action::{Action, KanIntent};
use crate::agent::PlayerView;
use crate::hand::{Hand, Meld};
use crate::tile::Tile;

use super::shanten::hand_from_parts;

pub fn prefer_win(legal: &[Action]) -> Option<Action> {
    legal
        .iter()
        .copied()
        .find(|action| matches!(action, Action::Tsumo | Action::Ron))
}

pub fn hand_from_view(view: &PlayerView) -> Option<Hand> {
    hand_from_parts(
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

fn remove_matching_identity(concealed: &mut Vec<Tile>, tile: Tile, count: usize) -> Option<()> {
    for _ in 0..count {
        let pos = concealed.iter().position(|t| t.matches_identity(tile))?;
        concealed.remove(pos);
    }
    Some(())
}
