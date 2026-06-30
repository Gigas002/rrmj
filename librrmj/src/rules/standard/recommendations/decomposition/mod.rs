#[cfg(test)]
mod tests;

use crate::rules::profile::WinContext;
use crate::rules::recommendations::{PathDecomposition, PathGroup};
use crate::rules::standard::patterns::{self, Decomposition, MeldComponent};
use crate::rules::standard::win::{
    all_tiles_in_hand, is_chiitoitsu_hand, tiles_with_win_tile,
};
use crate::tile::Tile;

pub(crate) fn build_from_context(
    ctx: &WinContext<'_>,
    shanten: i8,
    suggested_discard: Option<Tile>,
) -> PathDecomposition {
    let missing = if shanten >= 0 {
        vec![ctx.win_tile]
    } else {
        Vec::new()
    };
    let groups = if is_chiitoitsu_hand(ctx.hand(), ctx.win_tile, ctx.win_type) {
        chiitoitsu_groups(ctx)
    } else if let Some(decomp) = pick_display_decomposition(ctx) {
        decomposition_groups(decomp)
    } else {
        fallback_groups(ctx)
    };
    PathDecomposition {
        groups,
        missing,
        suggested_discard: if shanten == 1 {
            suggested_discard
        } else {
            None
        },
    }
}

fn pick_display_decomposition(ctx: &WinContext<'_>) -> Option<Decomposition> {
    let mut decomps = patterns::decompositions(ctx);
    if decomps.is_empty() {
        return None;
    }
    decomps.sort_by_cached_key(decomposition_key);
    decomps.into_iter().next()
}

fn decomposition_key(decomp: &Decomposition) -> String {
    let mut meld_keys: Vec<String> = decomp
        .melds
        .iter()
        .map(|meld| meld_component_label(*meld))
        .collect();
    meld_keys.sort();
    format!("{}:{}", decomp.pair, meld_keys.join("|"))
}

fn decomposition_groups(decomp: Decomposition) -> Vec<PathGroup> {
    let mut groups = Vec::with_capacity(5);
    for meld in decomp.melds {
        groups.push(PathGroup {
            tiles: meld.tiles(),
            open: meld.open,
        });
    }
    groups.push(PathGroup {
        tiles: vec![decomp.pair, decomp.pair],
        open: false,
    });
    groups
}

fn chiitoitsu_groups(ctx: &WinContext<'_>) -> Vec<PathGroup> {
    let mut tiles = all_tiles_in_hand(ctx.hand(), ctx.win_tile, ctx.win_type);
    tiles.sort_by(|a, b| a.cmp_sort(*b));
    tiles
        .chunks(2)
        .map(|pair| PathGroup {
            tiles: pair.to_vec(),
            open: false,
        })
        .collect()
}

fn fallback_groups(ctx: &WinContext<'_>) -> Vec<PathGroup> {
    let hand = ctx.hand();
    let mut groups = Vec::new();
    for meld in hand.melds() {
        groups.push(PathGroup {
            tiles: meld.tiles().to_vec(),
            open: true,
        });
    }
    let mut concealed = tiles_with_win_tile(hand, ctx.win_tile, ctx.win_type);
    concealed.sort_by(|a, b| a.cmp_sort(*b));
    if !concealed.is_empty() {
        groups.push(PathGroup {
            tiles: concealed,
            open: false,
        });
    }
    groups
}

fn meld_component_label(meld: MeldComponent) -> String {
    let mut tiles = meld.tiles();
    tiles.sort_by(|a, b| a.cmp_sort(*b));
    let body: String = tiles.iter().map(|t| t.to_string()).collect();
    if meld.open {
        format!("[{body}]")
    } else {
        body
    }
}
