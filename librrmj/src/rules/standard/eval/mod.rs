//! Yaku and fu evaluation over every reading of a winning hand.
//!
//! A complete hand can often be read more than one way: different splits into
//! sets and pair, and (for the standard form) different groups the winning
//! tile may have completed. Each reading gets its own yaku list and fu; the
//! reading that scores the most points wins (then more han, then more fu).

#[cfg(test)]
mod fu_tests;
#[cfg(test)]
mod tests;

use crate::hand::{KanForm, MeldKind};
use crate::rules::RulesConfig;
use crate::rules::profile::WinContext;
use crate::scoring::limits::base_points;
use crate::scoring::{WinType, Yaku};
use crate::tile::{Dragon, Suit, Tile, TileKind, Wind};

use super::win::{is_terminal_or_honor, tiles_with_win_tile};
use super::yaku::{is_kokushi_tiles, total_han};

/// The best reading of a winning hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Evaluation {
    pub yaku: Vec<Yaku>,
    pub fu: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// Lowest tile of a run.
    Sequence(Suit, u8),
    Triplet(Tile),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Group {
    shape: Shape,
    /// Formed from concealed tiles (or a closed kan).
    concealed: bool,
    kan: bool,
}

/// Which part of the hand the winning tile completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wait {
    Tanki,
    Ryanmen(usize),
    Kanchan(usize),
    Penchan(usize),
    Shanpon(usize),
}

impl Wait {
    const fn fu(self) -> u8 {
        match self {
            Self::Tanki | Self::Kanchan(_) | Self::Penchan(_) => 2,
            Self::Ryanmen(_) | Self::Shanpon(_) => 0,
        }
    }
}

struct Table {
    seat_wind: Tile,
    round_wind: Tile,
    menzen: bool,
    ron: bool,
    kiriage: bool,
    extra_han: u8,
}

/// Evaluates every reading of the winning hand and returns the best one with
/// at least one yaku. `extra_han` (dora) only breaks ties between limit
/// bands; it never makes a yakuless reading count. `None` when no reading has
/// a yaku (or the tiles don't form a winning hand).
pub(crate) fn evaluate(
    ctx: &WinContext<'_>,
    config: &RulesConfig,
    extra_han: u8,
) -> Option<Evaluation> {
    let table = Table {
        seat_wind: seat_wind(ctx.winner, ctx.state.dealer()),
        round_wind: ctx.state.round_wind_tile(),
        menzen: ctx.is_menzen(),
        ron: matches!(ctx.win_type, WinType::Ron { .. }),
        kiriage: config.kiriage,
        extra_han,
    };
    let situational = situational_yaku(ctx);
    let mut concealed = tiles_with_win_tile(ctx.hand(), ctx.win_tile, ctx.win_type);
    concealed.sort();
    let all_tiles = {
        let mut tiles = concealed.clone();
        for meld in ctx.hand().melds() {
            tiles.extend(meld.tiles().iter().map(|t| t.plain()));
        }
        tiles
    };
    let win_tile = ctx.win_tile.plain();

    let mut best: Option<Evaluation> = None;
    let mut consider = |yaku: Vec<Yaku>, fu: u8| {
        if yaku.is_empty() {
            return;
        }
        let candidate = Evaluation { yaku, fu };
        if best
            .as_ref()
            .is_none_or(|current| rank(&candidate, &table) > rank(current, &table))
        {
            best = Some(candidate);
        }
    };

    if table.menzen && ctx.hand().melds().is_empty() {
        if is_kokushi_tiles(&concealed) {
            let mut yaku = yakuman_only(&situational);
            yaku.push(Yaku::Kokushi);
            consider(yaku, 30);
        }
        if is_seven_pairs(&concealed) {
            let (yaku, fu) = chiitoitsu_reading(&situational, &all_tiles);
            consider(yaku, fu);
        }
    }

    let open_groups = called_groups(ctx);
    let sets_needed = 4usize.saturating_sub(open_groups.len());
    for (pair, sets) in standard_splits(&concealed, sets_needed) {
        let mut groups = open_groups.clone();
        groups.extend(sets.iter().map(|&shape| Group {
            shape,
            concealed: true,
            kan: false,
        }));
        for wait in waits(&groups, pair, win_tile) {
            let (yaku, fu) =
                standard_reading(&table, &situational, &groups, pair, wait, &all_tiles);
            consider(yaku, fu);
        }
    }

    best
}

fn rank(eval: &Evaluation, table: &Table) -> (i32, u8, u8) {
    let han = total_han(&eval.yaku, !table.menzen).saturating_add(table.extra_han);
    (base_points(han, eval.fu, table.kiriage), han, eval.fu)
}

/// Yaku that depend on timing and declarations, not on the tiles.
fn situational_yaku(ctx: &WinContext<'_>) -> Vec<Yaku> {
    let mut yaku = Vec::new();
    if ctx.state.is_double_riichi(ctx.winner) {
        yaku.push(Yaku::DoubleRiichi);
    } else if ctx.is_riichi() {
        yaku.push(Yaku::Riichi);
    }
    if ctx.is_riichi() && ctx.state.ippatsu_live(ctx.winner) {
        yaku.push(Yaku::Ippatsu);
    }
    if ctx.is_tenhou() || ctx.is_chiihou() {
        yaku.push(Yaku::TenhouChiihou);
        return yaku;
    }
    if matches!(ctx.win_type, WinType::Tsumo) && ctx.is_menzen() {
        yaku.push(Yaku::MenzenTsumo);
    }
    if ctx.is_haitei || ctx.is_houtei {
        yaku.push(Yaku::HaiteiHoutei);
    }
    if ctx.is_rinshan {
        yaku.push(Yaku::Rinshan);
    }
    if ctx.is_chankan {
        yaku.push(Yaku::Chankan);
    }
    if ctx.is_renhou() {
        yaku.push(Yaku::Renhou);
    }
    yaku
}

fn yakuman_only(yaku: &[Yaku]) -> Vec<Yaku> {
    yaku.iter().copied().filter(|y| y.is_yakuman()).collect()
}

/// A yakuman reading keeps only yakuman; otherwise the regular list.
fn finish(situational: &[Yaku], mut regular: Vec<Yaku>, mut yakuman: Vec<Yaku>) -> Vec<Yaku> {
    let timing_yakuman = yakuman_only(situational);
    suppress_subsumed(&mut yakuman);
    if !yakuman.is_empty() || !timing_yakuman.is_empty() {
        let mut out = timing_yakuman;
        out.append(&mut yakuman);
        return out;
    }
    let mut out = situational.to_vec();
    out.append(&mut regular);
    out
}

/// Keep the more specific limit hand where yakuman overlap (see RULES.md).
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

fn chiitoitsu_reading(situational: &[Yaku], tiles: &[Tile]) -> (Vec<Yaku>, u8) {
    let mut yakuman = Vec::new();
    if tiles.iter().all(|t| is_honor(*t)) {
        yakuman.push(Yaku::Tsuuiisou);
    }
    let mut regular = vec![Yaku::Chiitoitsu];
    regular.extend(tile_set_yaku(tiles));
    (finish(situational, regular, yakuman), 25)
}

/// Yaku decided by which tiles are in the hand, whatever the reading.
fn tile_set_yaku(tiles: &[Tile]) -> Vec<Yaku> {
    let mut yaku = Vec::new();
    if tiles.iter().all(|t| is_simple(*t)) {
        yaku.push(Yaku::Tanyao);
    }
    if let Some(flush) = flush(tiles) {
        yaku.push(flush);
    }
    if tiles.iter().all(|t| is_terminal_or_honor(*t))
        && tiles.iter().any(|t| is_honor(*t))
        && tiles.iter().any(|t| !is_honor(*t))
    {
        yaku.push(Yaku::Honroutou);
    }
    yaku
}

fn flush(tiles: &[Tile]) -> Option<Yaku> {
    let mut suits = tiles.iter().filter_map(|t| t.suit());
    let first = suits.next()?;
    if !suits.all(|s| s == first) {
        return None;
    }
    if tiles.iter().any(|t| is_honor(*t)) {
        Some(Yaku::Honitsu)
    } else {
        Some(Yaku::Chinitsu)
    }
}

fn standard_reading(
    table: &Table,
    situational: &[Yaku],
    groups: &[Group],
    pair: Tile,
    wait: Wait,
    all_tiles: &[Tile],
) -> (Vec<Yaku>, u8) {
    // A triplet completed by ron counts as open (fu, sanankou, suuankou).
    let ron_triplet = match wait {
        Wait::Shanpon(index) if table.ron => Some(index),
        _ => None,
    };
    let concealed_triplets = groups
        .iter()
        .enumerate()
        .filter(|(i, g)| {
            matches!(g.shape, Shape::Triplet(_)) && g.concealed && Some(*i) != ron_triplet
        })
        .count();
    let triplets: Vec<Tile> = groups
        .iter()
        .filter_map(|g| match g.shape {
            Shape::Triplet(t) => Some(t),
            Shape::Sequence(..) => None,
        })
        .collect();
    let sequences: Vec<(Suit, u8)> = groups
        .iter()
        .filter_map(|g| match g.shape {
            Shape::Sequence(s, r) => Some((s, r)),
            Shape::Triplet(_) => None,
        })
        .collect();
    let kans = groups.iter().filter(|g| g.kan).count();
    let dragon_triplets = triplets.iter().filter(|t| is_dragon(**t)).count();
    let wind_triplets = triplets.iter().filter(|t| is_wind(**t)).count();

    let mut yakuman = Vec::new();
    if concealed_triplets == 4 {
        yakuman.push(Yaku::Suuankou);
    }
    if dragon_triplets == 3 {
        yakuman.push(Yaku::Daisangen);
    }
    if wind_triplets == 4 {
        yakuman.push(Yaku::Daisuushii);
    } else if wind_triplets == 3 && is_wind(pair) {
        yakuman.push(Yaku::Shousuushii);
    }
    if all_tiles.iter().all(|t| is_honor(*t)) {
        yakuman.push(Yaku::Tsuuiisou);
    }
    if all_tiles.iter().all(|t| is_numbered_terminal(*t)) {
        yakuman.push(Yaku::Chinroutou);
    }
    if all_tiles.iter().all(|t| is_green(*t)) {
        yakuman.push(Yaku::Ryuuiisou);
    }
    if table.menzen && kans == 0 && is_chuuren(all_tiles) {
        yakuman.push(Yaku::Chuuren);
    }
    if kans == 4 {
        yakuman.push(Yaku::Suukantsu);
    }

    let pair_fu = pair_fu(pair, table);
    let pinfu =
        table.menzen && sequences.len() == 4 && pair_fu == 0 && matches!(wait, Wait::Ryanmen(_));

    let mut regular = tile_set_yaku(all_tiles);
    if pinfu {
        regular.push(Yaku::Pinfu);
    }
    for &tile in &triplets {
        if is_dragon(tile) {
            regular.push(Yaku::Yakuhai);
        }
        if tile == table.seat_wind {
            regular.push(Yaku::Yakuhai);
        }
        if tile == table.round_wind {
            regular.push(Yaku::Yakuhai);
        }
    }
    if dragon_triplets == 2 && is_dragon(pair) {
        regular.push(Yaku::Shousangen);
    }
    if table.menzen {
        match identical_sequence_pairs(&sequences) {
            2 => regular.push(Yaku::Ryanpeikou),
            1 => regular.push(Yaku::Iipeikou),
            _ => {}
        }
    }
    if sanshoku(&sequences) {
        regular.push(Yaku::Sanshoku);
    }
    if ittsu(&sequences) {
        regular.push(Yaku::Ittsu);
    }
    if sanshoku_doukou(&triplets) {
        regular.push(Yaku::SanshokuDoukou);
    }
    if !sequences.is_empty()
        && groups.iter().all(|g| group_has_terminal_or_honor(g.shape))
        && is_terminal_or_honor(pair)
    {
        if all_tiles.iter().any(|t| is_honor(*t)) {
            regular.push(Yaku::Chanta);
        } else {
            regular.push(Yaku::Junchan);
        }
    }
    if triplets.len() == 4 {
        regular.push(Yaku::Toitoi);
    }
    if concealed_triplets == 3 {
        regular.push(Yaku::Sanankou);
    }
    if kans == 3 {
        regular.push(Yaku::Sankantsu);
    }

    let yaku = finish(situational, regular, yakuman);
    let fu = if yaku.iter().any(|y| y.is_yakuman()) {
        30
    } else if pinfu {
        if table.ron { 30 } else { 20 }
    } else {
        standard_fu(table, groups, pair_fu, wait, ron_triplet)
    };
    (yaku, fu)
}

fn standard_fu(
    table: &Table,
    groups: &[Group],
    pair_fu: u8,
    wait: Wait,
    ron_triplet: Option<usize>,
) -> u8 {
    let mut fu = 20u8;
    for (index, group) in groups.iter().enumerate() {
        let Shape::Triplet(tile) = group.shape else {
            continue;
        };
        let mut meld_fu = if is_terminal_or_honor(tile) { 4 } else { 2 };
        if group.concealed && Some(index) != ron_triplet {
            meld_fu *= 2;
        }
        if group.kan {
            meld_fu *= 4;
        }
        fu += meld_fu;
    }
    fu += pair_fu + wait.fu();
    if table.menzen && table.ron {
        fu += 10;
    }
    if !table.ron {
        fu += 2;
    }
    let fu = fu.div_ceil(10) * 10;
    // An open hand with no fu at all (kui-pinfu) still scores 30.
    fu.max(30)
}

/// Dragon or seat/round wind pair: +2 each, so a double-wind pair is +4.
fn pair_fu(pair: Tile, table: &Table) -> u8 {
    let mut fu = 0;
    if is_dragon(pair) {
        fu += 2;
    }
    if pair == table.seat_wind {
        fu += 2;
    }
    if pair == table.round_wind {
        fu += 2;
    }
    fu
}

fn called_groups(ctx: &WinContext<'_>) -> Vec<Group> {
    ctx.hand()
        .melds()
        .iter()
        .filter_map(|meld| {
            let mut tiles: Vec<Tile> = meld.tiles().iter().map(|t| t.plain()).collect();
            tiles.sort();
            let shape = match meld.kind() {
                MeldKind::Chi => Shape::Sequence(tiles[0].suit()?, tiles[0].rank()?),
                MeldKind::Pon | MeldKind::Kan(_) => Shape::Triplet(tiles[0]),
            };
            Some(Group {
                shape,
                concealed: meld.kind() == MeldKind::Kan(KanForm::Closed),
                kan: matches!(meld.kind(), MeldKind::Kan(_)),
            })
        })
        .collect()
}

/// Every (pair, sets) split of the concealed tiles.
fn standard_splits(tiles: &[Tile], sets_needed: usize) -> Vec<(Tile, Vec<Shape>)> {
    let mut out = Vec::new();
    if tiles.len() != sets_needed * 3 + 2 {
        return out;
    }
    let mut pairs: Vec<Tile> = Vec::new();
    for window in tiles.windows(2) {
        if window[0] == window[1] && !pairs.contains(&window[0]) {
            pairs.push(window[0]);
        }
    }
    for pair in pairs {
        let mut rest = tiles.to_vec();
        for _ in 0..2 {
            let pos = rest.iter().position(|t| *t == pair).expect("pair tile");
            rest.remove(pos);
        }
        let mut current = Vec::new();
        split_sets(&rest, &mut current, &mut |sets| {
            out.push((pair, sets.to_vec()))
        });
    }
    out
}

fn split_sets(tiles: &[Tile], current: &mut Vec<Shape>, emit: &mut dyn FnMut(&[Shape])) {
    let Some(&first) = tiles.first() else {
        emit(current);
        return;
    };
    if tiles.len() >= 3 && tiles[1] == first && tiles[2] == first {
        current.push(Shape::Triplet(first));
        split_sets(&tiles[3..], current, emit);
        current.pop();
    }
    if let (Some(suit), Some(rank)) = (first.suit(), first.rank())
        && rank <= 7
    {
        let mut rest = tiles[1..].to_vec();
        let mut ok = true;
        for next in [rank + 1, rank + 2] {
            match rest.iter().position(|t| *t == Tile::numbered(suit, next)) {
                Some(pos) => {
                    rest.remove(pos);
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            current.push(Shape::Sequence(suit, rank));
            split_sets(&rest, current, emit);
            current.pop();
        }
    }
}

/// Every way the winning tile can have completed this split.
fn waits(groups: &[Group], pair: Tile, win_tile: Tile) -> Vec<Wait> {
    let mut out = Vec::new();
    if pair == win_tile {
        out.push(Wait::Tanki);
    }
    for (index, group) in groups.iter().enumerate() {
        if !group.concealed || group.kan {
            continue;
        }
        match group.shape {
            Shape::Triplet(tile) if tile == win_tile => out.push(Wait::Shanpon(index)),
            Shape::Sequence(suit, low) if win_tile.suit() == Some(suit) => {
                let Some(rank) = win_tile.rank() else {
                    continue;
                };
                if rank == low + 1 {
                    out.push(Wait::Kanchan(index));
                } else if rank == low {
                    out.push(if low == 7 {
                        Wait::Penchan(index)
                    } else {
                        Wait::Ryanmen(index)
                    });
                } else if rank == low + 2 {
                    out.push(if low == 1 {
                        Wait::Penchan(index)
                    } else {
                        Wait::Ryanmen(index)
                    });
                }
            }
            _ => {}
        }
    }
    out
}

fn identical_sequence_pairs(sequences: &[(Suit, u8)]) -> usize {
    let mut sorted: Vec<(usize, u8)> = sequences
        .iter()
        .map(|(s, r)| (suit_index(*s), *r))
        .collect();
    sorted.sort_unstable();
    let mut pairs = 0;
    let mut i = 0;
    while i + 1 < sorted.len() {
        if sorted[i] == sorted[i + 1] {
            pairs += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    pairs
}

fn sanshoku(sequences: &[(Suit, u8)]) -> bool {
    (1..=7).any(|rank| {
        Suit::ALL
            .iter()
            .all(|suit| sequences.contains(&(*suit, rank)))
    })
}

fn ittsu(sequences: &[(Suit, u8)]) -> bool {
    Suit::ALL.iter().any(|suit| {
        [1, 4, 7]
            .iter()
            .all(|rank| sequences.contains(&(*suit, *rank)))
    })
}

fn sanshoku_doukou(triplets: &[Tile]) -> bool {
    (1..=9).any(|rank| {
        Suit::ALL
            .iter()
            .all(|suit| triplets.contains(&Tile::numbered(*suit, rank)))
    })
}

fn group_has_terminal_or_honor(shape: Shape) -> bool {
    match shape {
        Shape::Sequence(_, low) => low == 1 || low == 7,
        Shape::Triplet(tile) => is_terminal_or_honor(tile),
    }
}

/// Seven distinct pairs (four of a kind is not two pairs).
fn is_seven_pairs(tiles: &[Tile]) -> bool {
    tiles.len() == 14
        && tiles.chunks(2).all(|pair| pair[0] == pair[1])
        && tiles
            .chunks(2)
            .zip(tiles.chunks(2).skip(1))
            .all(|(a, b)| a[0] != b[0])
}

fn is_chuuren(tiles: &[Tile]) -> bool {
    let Some(suit) = tiles.first().and_then(|t| t.suit()) else {
        return false;
    };
    let mut counts = [0u8; 9];
    for tile in tiles {
        if tile.suit() != Some(suit) {
            return false;
        }
        counts[usize::from(tile.rank().unwrap_or(1) - 1)] += 1;
    }
    tiles.len() == 14 && counts[0] >= 3 && counts[8] >= 3 && counts[1..8].iter().all(|&c| c >= 1)
}

fn seat_wind(seat: usize, dealer: usize) -> Tile {
    Tile::wind(Wind::ALL[(seat + 4 - dealer) % 4])
}

fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Man => 0,
        Suit::Pin => 1,
        Suit::Sou => 2,
    }
}

fn is_simple(tile: Tile) -> bool {
    matches!(tile.rank(), Some(2..=8))
}

fn is_honor(tile: Tile) -> bool {
    matches!(tile.kind(), TileKind::Wind(_) | TileKind::Dragon(_))
}

fn is_wind(tile: Tile) -> bool {
    matches!(tile.kind(), TileKind::Wind(_))
}

fn is_dragon(tile: Tile) -> bool {
    matches!(tile.kind(), TileKind::Dragon(_))
}

fn is_numbered_terminal(tile: Tile) -> bool {
    matches!(tile.rank(), Some(1 | 9))
}

fn is_green(tile: Tile) -> bool {
    matches!(
        tile.kind(),
        TileKind::Sou(2 | 3 | 4 | 6 | 8) | TileKind::Dragon(Dragon::Green)
    )
}

/// Fu of the best reading (test helper for the fu table).
#[cfg(test)]
pub(crate) fn calculate_fu(ctx: &WinContext<'_>, config: &RulesConfig) -> u8 {
    evaluate(ctx, config, 0).map_or(30, |eval| eval.fu)
}
