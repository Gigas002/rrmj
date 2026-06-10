use crate::hand::{Concealed, Hand, MeldKind};
use crate::rules::profile_trait::WinContext;
use crate::tile::{Suit, Tile, TileKind, Wind};

pub fn is_winning_hand(hand: &Hand, win_tile: Option<Tile>) -> bool {
    let mut concealed = hand.concealed().tiles().to_vec();
    if let Some(tile) = win_tile
        && (concealed.len() < 14 || !concealed.contains(&tile))
    {
        concealed.push(tile);
    }
    concealed.sort();

    let open_sets = hand.melds().len();
    if open_sets > 4 {
        return false;
    }

    if hand.melds().is_empty() && is_chiitoitsu(&concealed) {
        return true;
    }

    if hand.melds().is_empty() && super::yakuman::is_kokushi_tiles(&concealed) {
        return true;
    }

    if hand.melds().is_empty() && is_chuuren_tiles(&concealed) {
        return true;
    }

    let sets_needed = 4usize.saturating_sub(open_sets);
    can_form_mentsu_and_pair(&concealed, sets_needed)
}

pub fn is_tenpai(hand: &Hand) -> bool {
    for tile in all_wait_tiles() {
        if is_winning_hand(hand, Some(tile)) {
            return true;
        }
    }
    false
}

/// Whether the hand stays tenpai after removing one tile from the concealed hand.
pub fn is_tenpai_after_discard(hand: &Hand, discard: Tile) -> bool {
    let mut concealed = hand.concealed().tiles().to_vec();
    let Some(pos) = concealed.iter().position(|&t| t == discard) else {
        return false;
    };
    concealed.remove(pos);
    let after = Hand::new(Concealed::from_tiles(concealed), hand.melds().to_vec())
        .expect("discard leaves valid hand shape");
    is_tenpai(&after)
}

pub fn is_chiitoitsu_hand(hand: &Hand, win_tile: Tile, win_type: crate::scoring::WinType) -> bool {
    if !hand.melds().is_empty() {
        return false;
    }
    let mut tiles = tiles_with_win_tile(hand, win_tile, win_type);
    tiles.sort();
    is_chiitoitsu(&tiles)
}

fn is_chuuren_tiles(tiles: &[Tile]) -> bool {
    crate::tile::Suit::ALL
        .iter()
        .any(|&suit| chuuren_suit_counts(tiles, suit).is_some())
}

fn chuuren_suit_counts(tiles: &[Tile], suit: crate::tile::Suit) -> Option<[u8; 9]> {
    let mut counts = [0u8; 9];
    for tile in tiles {
        let tile_suit = tile.suit()?;
        if tile_suit != suit {
            return None;
        }
        let rank = tile.rank()?;
        counts[(rank - 1) as usize] += 1;
    }
    if counts.iter().sum::<u8>() != 14 {
        return None;
    }
    if counts[0] >= 3 && counts[8] >= 3 && counts[1..8].iter().all(|&c| c >= 1) {
        Some(counts)
    } else {
        None
    }
}

fn is_chiitoitsu(tiles: &[Tile]) -> bool {
    if tiles.len() != 14 {
        return false;
    }
    tiles
        .chunks(2)
        .all(|pair| pair.len() == 2 && pair[0] == pair[1])
}

fn can_form_mentsu_and_pair(tiles: &[Tile], sets_needed: usize) -> bool {
    if sets_needed == 0 {
        return tiles.len() == 2 && tiles[0] == tiles[1];
    }

    for i in 0..tiles.len() {
        for j in (i + 1)..tiles.len() {
            if tiles[i] != tiles[j] {
                continue;
            }
            let mut rest = tiles.to_vec();
            rest.remove(j);
            rest.remove(i);
            if can_form_sets(&rest, sets_needed) {
                return true;
            }
        }
    }
    false
}

fn can_form_sets(tiles: &[Tile], sets_needed: usize) -> bool {
    if sets_needed == 0 {
        return tiles.is_empty();
    }
    if tiles.is_empty() {
        return false;
    }

    if tiles.len() >= 3
        && tiles[0] == tiles[1]
        && tiles[1] == tiles[2]
        && can_form_sets(&tiles[3..], sets_needed - 1)
    {
        return true;
    }

    if let (Some(suit), Some(rank)) = (tiles[0].suit(), tiles[0].rank())
        && rank <= 7
    {
        let t2 = Tile::numbered(suit, rank + 1);
        let t3 = Tile::numbered(suit, rank + 2);
        if let Some(rest) = remove_tiles(tiles, &[tiles[0], t2, t3])
            && can_form_sets(&rest, sets_needed - 1)
        {
            return true;
        }
    }

    false
}

fn remove_tiles(tiles: &[Tile], to_remove: &[Tile]) -> Option<Vec<Tile>> {
    let mut rest = tiles.to_vec();
    for tile in to_remove {
        let pos = rest.iter().position(|t| *t == *tile)?;
        rest.remove(pos);
    }
    Some(rest)
}

fn all_wait_tiles() -> Vec<Tile> {
    let mut tiles = Vec::new();
    for suit in Suit::ALL {
        for rank in 1..=9 {
            tiles.push(Tile::numbered(suit, rank));
            if rank == 5 {
                tiles.push(Tile::red_five(suit));
            }
        }
    }
    for wind in crate::tile::Wind::ALL {
        tiles.push(Tile::wind(wind));
    }
    for dragon in crate::tile::Dragon::ALL {
        tiles.push(Tile::dragon(dragon));
    }
    tiles
}

pub fn is_simple(tile: Tile) -> bool {
    matches!(tile.rank(), Some(2..=8))
}

pub fn is_terminal_or_honor(tile: Tile) -> bool {
    match tile.kind() {
        TileKind::Man(r) | TileKind::Pin(r) | TileKind::Sou(r) => r == 1 || r == 9,
        TileKind::Wind(_) | TileKind::Dragon(_) => true,
    }
}

pub fn triplet_fu(tile: Tile, closed: bool) -> u8 {
    let simple = !is_terminal_or_honor(tile);
    match (closed, simple) {
        (true, true) => 4,
        (true, false) => 8,
        (false, true) => 2,
        (false, false) => 4,
    }
}

/// Concealed meld, pair, and wait fu from the minimum-fu winning decomposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuDecomposition {
    pub concealed_meld_fu: u8,
    pub pair_fu: u8,
    pub wait_fu: u8,
}

pub fn best_fu_decomposition(ctx: &WinContext<'_>) -> FuDecomposition {
    let mut tiles = tiles_with_win_tile(ctx.hand(), ctx.win_tile, ctx.win_type);
    tiles.sort();

    let open_sets = ctx.hand().melds().len();
    let sets_needed = 4usize.saturating_sub(open_sets);
    let seat_wind = seat_wind_tile(ctx.winner, ctx.state.dealer());
    let round_wind = Tile::wind(Wind::East);

    decompose_for_fu(&tiles, sets_needed, ctx.win_tile, seat_wind, round_wind).unwrap_or(
        FuDecomposition {
            concealed_meld_fu: 0,
            pair_fu: 0,
            wait_fu: 0,
        },
    )
}

fn decompose_for_fu(
    tiles: &[Tile],
    sets_needed: usize,
    win_tile: Tile,
    seat_wind: Tile,
    round_wind: Tile,
) -> Option<FuDecomposition> {
    if sets_needed == 0 {
        if tiles.len() != 2 || tiles[0] != tiles[1] {
            return None;
        }
        let pair_fu = valued_pair_fu(tiles[0], seat_wind, round_wind);
        let wait_fu = if tiles[0].identity() == win_tile.identity() {
            2
        } else {
            0
        };
        return Some(FuDecomposition {
            concealed_meld_fu: 0,
            pair_fu,
            wait_fu,
        });
    }

    let mut best: Option<FuDecomposition> = None;

    for i in 0..tiles.len() {
        for j in (i + 1)..tiles.len() {
            if tiles[i] != tiles[j] {
                continue;
            }
            let mut rest = tiles.to_vec();
            rest.remove(j);
            rest.remove(i);
            let pair_fu = valued_pair_fu(tiles[i], seat_wind, round_wind);
            if let Some(mut child) = decompose_sets_for_fu(&rest, sets_needed, win_tile) {
                child.pair_fu = pair_fu;
                if tiles[i].identity() == win_tile.identity() {
                    child.wait_fu = child.wait_fu.max(2);
                }
                best = Some(min_fu_decomp(best, child));
            }
        }
    }

    best
}

fn decompose_sets_for_fu(
    tiles: &[Tile],
    sets_needed: usize,
    win_tile: Tile,
) -> Option<FuDecomposition> {
    if sets_needed == 0 {
        return if tiles.is_empty() {
            Some(FuDecomposition {
                concealed_meld_fu: 0,
                pair_fu: 0,
                wait_fu: 0,
            })
        } else {
            None
        };
    }
    if tiles.is_empty() {
        return None;
    }

    let mut best: Option<FuDecomposition> = None;

    if tiles.len() >= 3 && tiles[0] == tiles[1] && tiles[1] == tiles[2] {
        let triplet_tile = tiles[0];
        let meld_fu = triplet_fu(triplet_tile, true);
        let wait_fu = if triplet_tile.identity() == win_tile.identity() {
            2
        } else {
            0
        };
        if let Some(mut child) = decompose_sets_for_fu(&tiles[3..], sets_needed - 1, win_tile) {
            child.concealed_meld_fu = child.concealed_meld_fu.saturating_add(meld_fu);
            child.wait_fu = child.wait_fu.max(wait_fu);
            best = Some(min_fu_decomp(best, child));
        }
    }

    if let (Some(suit), Some(rank)) = (tiles[0].suit(), tiles[0].rank())
        && rank <= 7
    {
        let t2 = Tile::numbered(suit, rank + 1);
        let t3 = Tile::numbered(suit, rank + 2);
        let meld = [tiles[0], t2, t3];
        if let Some(rest) = remove_tiles(tiles, &meld) {
            let wait_fu = sequence_wait_fu(&meld, win_tile);
            if let Some(mut child) = decompose_sets_for_fu(&rest, sets_needed - 1, win_tile) {
                child.wait_fu = child.wait_fu.max(wait_fu);
                best = Some(min_fu_decomp(best, child));
            }
        }
    }

    best
}

fn min_fu_decomp(current: Option<FuDecomposition>, candidate: FuDecomposition) -> FuDecomposition {
    match current {
        None => candidate,
        Some(prev) => {
            let prev_cost = prev.concealed_meld_fu + prev.wait_fu;
            let cand_cost = candidate.concealed_meld_fu + candidate.wait_fu;
            if cand_cost < prev_cost {
                candidate
            } else {
                prev
            }
        }
    }
}

fn valued_pair_fu(pair: Tile, seat_wind: Tile, round_wind: Tile) -> u8 {
    if pair == seat_wind || pair == round_wind || matches!(pair.kind(), TileKind::Dragon(_)) {
        2
    } else {
        0
    }
}

fn sequence_wait_fu(meld: &[Tile; 3], win_tile: Tile) -> u8 {
    if !meld.contains(&win_tile) {
        return 0;
    }
    if is_ryanmen_sequence_win(meld, win_tile) {
        0
    } else {
        2
    }
}

pub fn hand_contains_yakuhai(hand: &Hand, tile: Tile) -> bool {
    let identity = tile.identity();
    hand.concealed()
        .tiles()
        .iter()
        .any(|t| t.identity() == identity)
        || hand.melds().iter().any(|meld| {
            meld.tiles()
                .iter()
                .any(|t| t.identity() == identity && meld.kind() != MeldKind::Chi)
        })
}

pub fn all_tiles_in_hand(
    hand: &Hand,
    win_tile: Tile,
    win_type: crate::scoring::WinType,
) -> Vec<Tile> {
    let mut tiles = tiles_with_win_tile(hand, win_tile, win_type);
    for meld in hand.melds() {
        tiles.extend_from_slice(meld.tiles());
    }
    tiles.sort();
    tiles
}

/// Concealed tiles plus the winning tile (ron always adds the called tile).
pub(crate) fn tiles_with_win_tile(
    hand: &Hand,
    win_tile: Tile,
    win_type: crate::scoring::WinType,
) -> Vec<Tile> {
    let mut tiles = hand.concealed().tiles().to_vec();
    match win_type {
        crate::scoring::WinType::Ron { .. } if tiles.len() < 14 => tiles.push(win_tile),
        crate::scoring::WinType::Tsumo if !tiles.contains(&win_tile) => tiles.push(win_tile),
        _ => {}
    }
    tiles
}

pub fn is_tanyao_hand(hand: &Hand, win_tile: Tile, win_type: crate::scoring::WinType) -> bool {
    all_tiles_in_hand(hand, win_tile, win_type)
        .iter()
        .all(|tile| is_simple(*tile))
}

pub fn is_pinfu_hand(ctx: &WinContext<'_>) -> bool {
    if !ctx.is_menzen() || !is_winning_hand(ctx.hand(), Some(ctx.win_tile)) {
        return false;
    }

    let tiles = all_tiles_in_hand(ctx.hand(), ctx.win_tile, ctx.win_type);
    if tiles.len() != 14 {
        return false;
    }

    let seat_wind = seat_wind_tile(ctx.winner, ctx.state.dealer());
    let round_wind = Tile::wind(Wind::East);

    for i in 0..tiles.len() {
        for j in (i + 1)..tiles.len() {
            if tiles[i] != tiles[j] {
                continue;
            }
            let pair = tiles[i];
            if is_scoring_pair(pair, seat_wind, round_wind) {
                continue;
            }
            let mut rest = tiles.clone();
            rest.remove(j);
            rest.remove(i);
            rest.sort();
            if pinfu_sequence_decomposition(&rest, 4, ctx.win_tile) {
                return true;
            }
        }
    }
    false
}

fn is_scoring_pair(tile: Tile, seat_wind: Tile, round_wind: Tile) -> bool {
    tile == seat_wind
        || tile == round_wind
        || matches!(tile.kind(), crate::tile::TileKind::Dragon(_))
}

fn seat_wind_tile(seat: usize, dealer: usize) -> Tile {
    let index = (seat + 4 - dealer) % 4;
    Tile::wind(Wind::ALL[index])
}

fn pinfu_sequence_decomposition(tiles: &[Tile], sequences_needed: usize, win_tile: Tile) -> bool {
    decompose_pinfu_sequences(tiles, sequences_needed, win_tile, false)
}

fn decompose_pinfu_sequences(
    tiles: &[Tile],
    sequences_needed: usize,
    win_tile: Tile,
    found_ryanmen: bool,
) -> bool {
    if sequences_needed == 0 {
        return tiles.is_empty() && found_ryanmen;
    }
    if tiles.len() < 3 {
        return false;
    }

    if let (Some(suit), Some(rank)) = (tiles[0].suit(), tiles[0].rank())
        && rank <= 7
    {
        let t2 = Tile::numbered(suit, rank + 1);
        let t3 = Tile::numbered(suit, rank + 2);
        let meld = [tiles[0], t2, t3];
        if let Some(rest) = remove_tiles(tiles, &meld) {
            let win_in_meld = meld.contains(&win_tile);
            let ryanmen =
                found_ryanmen || (win_in_meld && is_ryanmen_sequence_win(&meld, win_tile));
            if decompose_pinfu_sequences(&rest, sequences_needed - 1, win_tile, ryanmen) {
                return true;
            }
        }
    }

    false
}

fn is_ryanmen_sequence_win(meld: &[Tile; 3], win_tile: Tile) -> bool {
    if !meld.contains(&win_tile) {
        return false;
    }
    let Some(win_rank) = win_tile.rank() else {
        return false;
    };
    let mut ranks: Vec<u8> = meld.iter().filter_map(|t| t.rank()).collect();
    ranks.sort_unstable();
    if ranks.len() != 3 {
        return false;
    }
    if win_rank == ranks[1] {
        return false;
    }
    if win_rank == ranks[2] {
        return win_rank > 3;
    }
    win_rank < 7
}
