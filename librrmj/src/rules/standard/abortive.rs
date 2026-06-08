use crate::game::{AbortiveDrawKind, AbortiveTrigger};
use crate::rules::RulesConfig;
use crate::state::HandState;
use crate::tile::{Tile, TileKind};

pub fn detect_abortive(
    state: &HandState,
    config: &RulesConfig,
    trigger: AbortiveTrigger,
) -> Option<AbortiveDrawKind> {
    match trigger {
        AbortiveTrigger::DealerFirstTurn if config.abortive_nine_terminals => nine_terminals(state),
        AbortiveTrigger::FirstDiscard { seat, tile } if config.abortive_four_winds => {
            four_winds(state, seat, tile)
        }
        AbortiveTrigger::KanDeclared if config.abortive_four_kongs => four_kongs(state),
        AbortiveTrigger::RiichiDeclared if config.abortive_four_riichis => four_riichis(state),
        _ => None,
    }
}

fn nine_terminals(state: &HandState) -> Option<AbortiveDrawKind> {
    if state.current_actor() != state.dealer() {
        return None;
    }
    let dealer = state.dealer();
    let hand = state.hand(dealer);
    if !hand.melds().is_empty() {
        return None;
    }
    let mut kinds = std::collections::HashSet::new();
    for tile in hand.concealed().tiles() {
        if is_terminal_or_honor(*tile) {
            kinds.insert(tile.identity());
        }
    }
    if kinds.len() >= 9 {
        Some(AbortiveDrawKind::NineTerminals)
    } else {
        None
    }
}

fn four_winds(state: &HandState, _seat: usize, tile: Tile) -> Option<AbortiveDrawKind> {
    let TileKind::Wind(_) = tile.kind() else {
        return None;
    };
    for seat in 0..4 {
        let first = state.first_discard(seat)?;
        if !first.matches_identity(tile) {
            return None;
        }
    }
    Some(AbortiveDrawKind::FourWinds)
}

fn four_kongs(state: &HandState) -> Option<AbortiveDrawKind> {
    if state.wall().kan_count() >= 4 {
        Some(AbortiveDrawKind::FourKongs)
    } else {
        None
    }
}

fn four_riichis(state: &HandState) -> Option<AbortiveDrawKind> {
    if (0..4).all(|seat| state.is_riichi(seat)) {
        Some(AbortiveDrawKind::FourRiichis)
    } else {
        None
    }
}

fn is_terminal_or_honor(tile: Tile) -> bool {
    match tile.kind() {
        TileKind::Man(r) | TileKind::Pin(r) | TileKind::Sou(r) => r == 1 || r == 9,
        TileKind::Wind(_) | TileKind::Dragon(_) => true,
    }
}
