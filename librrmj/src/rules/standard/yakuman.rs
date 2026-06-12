//! Yakuman (limit-hand) detection for the standard profile.

use crate::hand::{KanForm, MeldKind};
use crate::rules::profile_trait::WinContext;
use crate::rules::standard::patterns;
use crate::rules::standard::win::{all_tiles_in_hand, tiles_with_win_tile};
use crate::scoring::Yaku;
use crate::tile::{Dragon, Suit, Tile, TileIdentity, TileKind, Wind};

pub fn detect(ctx: &WinContext<'_>) -> Vec<Yaku> {
    let mut yaku = Vec::new();
    if is_kokushi_hand(ctx) {
        yaku.push(Yaku::Kokushi);
    }
    if is_chuuren_hand(ctx) {
        yaku.push(Yaku::Chuuren);
    }
    if is_suuankou_hand(ctx) {
        yaku.push(Yaku::Suuankou);
    }
    if is_daisangen_hand(ctx) {
        yaku.push(Yaku::Daisangen);
    }
    if is_daisuushii_hand(ctx) {
        yaku.push(Yaku::Daisuushii);
    } else if is_shousuushii_hand(ctx) {
        yaku.push(Yaku::Shousuushii);
    }
    if is_ryuuiisou_hand(ctx) {
        yaku.push(Yaku::Ryuuiisou);
    }
    if is_suukantsu_hand(ctx) {
        yaku.push(Yaku::Suukantsu);
    }
    suppress_subsumed(&mut yaku);
    yaku
}

/// Suuankou overlaps several other yakuman; keep the more specific limit hand.
fn suppress_subsumed(yaku: &mut Vec<Yaku>) {
    if yaku.contains(&Yaku::Daisuushii) {
        yaku.retain(|y| *y != Yaku::Shousuushii && *y != Yaku::Suuankou);
    } else if yaku.contains(&Yaku::Shousuushii) {
        yaku.retain(|y| *y != Yaku::Suuankou);
    }
    if yaku.iter().any(|y| {
        matches!(
            y,
            Yaku::Daisangen | Yaku::Kokushi | Yaku::Chuuren | Yaku::Ryuuiisou
        )
    }) {
        yaku.retain(|y| *y != Yaku::Suuankou);
    }
}

pub fn is_kokushi_hand(ctx: &WinContext<'_>) -> bool {
    if !ctx.is_menzen() {
        return false;
    }
    is_kokushi_tiles(&tiles_with_win_tile(ctx.hand(), ctx.win_tile, ctx.win_type))
}

pub fn is_kokushi_tiles(tiles: &[Tile]) -> bool {
    if tiles.len() != 14 {
        return false;
    }
    let mut counts = std::collections::HashMap::new();
    for tile in tiles {
        *counts.entry(tile.identity()).or_insert(0u8) += 1;
    }
    if counts.len() != 13 {
        return false;
    }
    kokushi_identities()
        .iter()
        .all(|id| counts.get(id).copied().unwrap_or(0) >= 1)
        && counts.values().sum::<u8>() == 14
}

pub fn is_chuuren_hand(ctx: &WinContext<'_>) -> bool {
    if !ctx.is_menzen() {
        return false;
    }
    let tiles = all_tiles_in_hand(ctx.hand(), ctx.win_tile, ctx.win_type);
    Suit::ALL.iter().any(|&suit| is_chuuren_suit(&tiles, suit))
}

fn is_chuuren_suit(tiles: &[Tile], suit: Suit) -> bool {
    let mut counts = [0u8; 9];
    for tile in tiles {
        let Some(tile_suit) = tile.suit() else {
            return false;
        };
        if tile_suit != suit {
            return false;
        }
        let Some(rank) = tile.rank() else {
            return false;
        };
        counts[(rank - 1) as usize] += 1;
    }
    if counts.iter().sum::<u8>() != 14 {
        return false;
    }
    counts[0] >= 3 && counts[8] >= 3 && counts[1..8].iter().all(|&c| c >= 1)
}

pub fn is_suuankou_hand(ctx: &WinContext<'_>) -> bool {
    if ctx
        .hand()
        .melds()
        .iter()
        .any(|meld| !matches!(meld.kind(), MeldKind::Kan(KanForm::Closed)))
    {
        return false;
    }
    patterns::decompositions(ctx)
        .into_iter()
        .any(|d| d.melds.iter().all(|m| m.triplet.is_some()))
}

pub fn is_daisangen_hand(ctx: &WinContext<'_>) -> bool {
    let dragons = [
        Tile::dragon(Dragon::White),
        Tile::dragon(Dragon::Green),
        Tile::dragon(Dragon::Red),
    ];
    patterns::decompositions(ctx)
        .into_iter()
        .any(|d| dragons.iter().all(|dragon| has_triplet(&d, *dragon)))
}

pub fn is_shousuushii_hand(ctx: &WinContext<'_>) -> bool {
    patterns::decompositions(ctx)
        .into_iter()
        .any(|d| wind_triplet_count(&d) == 3 && matches!(d.pair.kind(), TileKind::Wind(_)))
}

pub fn is_daisuushii_hand(ctx: &WinContext<'_>) -> bool {
    patterns::decompositions(ctx)
        .into_iter()
        .any(|d| wind_triplet_count(&d) == 4)
}

pub fn is_ryuuiisou_hand(ctx: &WinContext<'_>) -> bool {
    all_tiles_in_hand(ctx.hand(), ctx.win_tile, ctx.win_type)
        .iter()
        .all(|tile| is_green_tile(*tile))
}

pub fn is_suukantsu_hand(ctx: &WinContext<'_>) -> bool {
    ctx.hand()
        .melds()
        .iter()
        .filter(|meld| {
            matches!(meld.kind(), MeldKind::Kan(_))
        })
        .count()
        >= 4
}

fn has_triplet(decomp: &patterns::Decomposition, tile: Tile) -> bool {
    decomp.melds.iter().any(|m| m.triplet == Some(tile))
}

fn wind_triplet_count(decomp: &patterns::Decomposition) -> usize {
    decomp
        .melds
        .iter()
        .filter(|m| {
            m.triplet
                .is_some_and(|t| matches!(t.kind(), TileKind::Wind(_)))
        })
        .count()
}

fn is_green_tile(tile: Tile) -> bool {
    matches!(
        tile.kind(),
        TileKind::Sou(2)
            | TileKind::Sou(3)
            | TileKind::Sou(4)
            | TileKind::Sou(6)
            | TileKind::Sou(8)
    ) || matches!(tile.kind(), TileKind::Dragon(Dragon::Green))
}

fn kokushi_identities() -> [TileIdentity; 13] {
    [
        TileIdentity::Numbered {
            suit: Suit::Man,
            rank: 1,
        },
        TileIdentity::Numbered {
            suit: Suit::Man,
            rank: 9,
        },
        TileIdentity::Numbered {
            suit: Suit::Pin,
            rank: 1,
        },
        TileIdentity::Numbered {
            suit: Suit::Pin,
            rank: 9,
        },
        TileIdentity::Numbered {
            suit: Suit::Sou,
            rank: 1,
        },
        TileIdentity::Numbered {
            suit: Suit::Sou,
            rank: 9,
        },
        TileIdentity::Wind(Wind::East),
        TileIdentity::Wind(Wind::South),
        TileIdentity::Wind(Wind::West),
        TileIdentity::Wind(Wind::North),
        TileIdentity::Dragon(Dragon::White),
        TileIdentity::Dragon(Dragon::Green),
        TileIdentity::Dragon(Dragon::Red),
    ]
}
