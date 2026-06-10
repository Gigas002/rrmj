//! Standard-profile yaku catalog — tracks which winning-hand types the engine scores.
//!
//! Consumed by win-combination tests, scenario tooling, and future rules UI.

#![allow(dead_code)]

/// One row in the standard yaku catalog.
#[derive(Debug, Clone, Copy)]
pub struct CheatsheetRow {
    pub id: &'static str,
    pub label: &'static str,
    pub implemented: bool,
}

/// Every distinct winning-hand / yaku category for standard Japanese riichi.
pub const ROWS: &[CheatsheetRow] = &[
    row("riichi", "Riichi", true),
    row("double_riichi", "Double riichi", true),
    row("ippatsu", "Ippatsu", true),
    row("menzen_tsumo", "Menzen tsumo", true),
    row("pinfu", "Pinfu", true),
    row("iipeikou", "Iipeikou", true),
    row("ryanpeikou", "Ryanpeikou", true),
    row("chiitoitsu", "Chiitoitsu", true),
    row("haitei", "Haitei / Houtei", true),
    row("rinshan", "Rinshan kaihou", true),
    row("chankan", "Chankan", true),
    row("renhou", "Renhou", true),
    row("tenhou", "Tenhou / Chiihou", true),
    row("yakuhai", "Yakuhai", true),
    row("tanyao", "Tanyao", true),
    row("sanshoku", "Sanshoku doujun", true),
    row("ittsu", "Ittsu", true),
    row("chanta", "Chanta", true),
    row("junchan", "Junchan", true),
    row("toitoi", "Toitoi", true),
    row("honitsu", "Honitsu", true),
    row("chinitsu", "Chinitsu", true),
    row("kokushi", "Kokushi musou", true),
    row("suuankou", "Suuankou", true),
    row("daisangen", "Daisangen", true),
    row("shousuushii", "Shousuushii", true),
    row("daisuushii", "Daisuushii", true),
    row("chuuren", "Chuuren poutou", true),
    row("ryuuiisou", "Ryuuiisou", true),
    row("suukantsu", "Suukantsu", true),
];

const fn row(id: &'static str, label: &'static str, implemented: bool) -> CheatsheetRow {
    CheatsheetRow {
        id,
        label,
        implemented,
    }
}

pub fn implemented_rows() -> impl Iterator<Item = &'static CheatsheetRow> {
    ROWS.iter().filter(|r| r.implemented)
}

pub fn not_implemented_rows() -> impl Iterator<Item = &'static CheatsheetRow> {
    ROWS.iter().filter(|r| !r.implemented)
}
