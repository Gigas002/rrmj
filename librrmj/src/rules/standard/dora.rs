use crate::rules::RulesConfig;
use crate::rules::WinContext;
use crate::rules::standard::win;
use crate::tile::{Dragon, Tile, TileKind, Wind};

pub fn count_dora(ctx: &WinContext<'_>, _config: &RulesConfig) -> u8 {
    let indicators = ctx.state.wall().dora_indicators();
    count_matching(&indicators, ctx)
}

pub fn count_ura_dora(ctx: &WinContext<'_>, _config: &RulesConfig) -> u8 {
    if !ctx.is_riichi() {
        return 0;
    }
    let indicators = ctx.state.wall().ura_dora_indicators();
    count_matching(&indicators, ctx)
}

pub fn count_aka_dora(ctx: &WinContext<'_>, config: &RulesConfig) -> u8 {
    if !config.aka_dora {
        return 0;
    }
    win::all_tiles_in_hand(ctx.hand(), ctx.win_tile)
        .iter()
        .filter(|tile| tile.is_red())
        .count() as u8
}

fn count_matching(indicators: &[Tile], ctx: &WinContext<'_>) -> u8 {
    let hand = win::all_tiles_in_hand(ctx.hand(), ctx.win_tile);
    indicators
        .iter()
        .filter_map(|indicator| dora_tile(*indicator))
        .map(|dora| hand.iter().filter(|t| t.matches_identity(dora)).count() as u8)
        .sum()
}

pub fn dora_tile(indicator: Tile) -> Option<Tile> {
    match indicator.kind() {
        TileKind::Man(r) | TileKind::Pin(r) | TileKind::Sou(r) => {
            let suit = indicator.suit()?;
            Some(Tile::numbered(suit, if r == 9 { 1 } else { r + 1 }))
        }
        TileKind::Wind(w) => {
            let next = match w {
                Wind::East => Wind::South,
                Wind::South => Wind::West,
                Wind::West => Wind::North,
                Wind::North => Wind::East,
            };
            Some(Tile::wind(next))
        }
        TileKind::Dragon(d) => {
            let next = match d {
                Dragon::White => Dragon::Green,
                Dragon::Green => Dragon::Red,
                Dragon::Red => Dragon::White,
            };
            Some(Tile::dragon(next))
        }
    }
}
