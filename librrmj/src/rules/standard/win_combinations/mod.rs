//! One fixture per baseline scoring yaku (Phase 11.4 cheatsheet rows).

#[cfg(test)]
use crate::hand::Meld;
#[cfg(test)]
use crate::scoring::{WinType, Yaku};
#[cfg(test)]
use crate::tile::Tile;

/// Table-driven win case for baseline yaku verification.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct WinCase {
    pub id: &'static str,
    pub cheatsheet_id: &'static str,
    pub concealed: Vec<Tile>,
    pub melds: Vec<Meld>,
    pub winner: usize,
    pub dealer: usize,
    pub win_type: WinType,
    pub win_tile: Tile,
    pub riichi: bool,
    pub ippatsu_live: bool,
    pub double_riichi: bool,
    /// When set, drain the live wall to this count before scoring.
    pub wall_live_remaining: Option<usize>,
    pub is_rinshan_draw: bool,
    pub is_dealer_first_turn: bool,
    pub live_draws: [u8; 4],
    pub calls_made: bool,
    pub is_chankan: bool,
    pub must_include: &'static [Yaku],
    pub must_exclude: &'static [Yaku],
    /// Sum of han from scored yaku (excludes dora).
    pub expected_yaku_han: u8,
    pub expected_fu: u8,
}

#[cfg(test)]
mod tests;
