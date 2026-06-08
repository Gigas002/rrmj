mod layout;

#[cfg(test)]
mod tests;

pub use layout::WallLayout;

use rand::Rng;
use rand::seq::SliceRandom;

use crate::Error;
use crate::hand::{DEALER_HAND_SIZE, Hand, NON_DEALER_HAND_SIZE, validate_deal_counts};
use crate::rules::RulesConfig;
use crate::tile::{Tile, standard_set};

pub const WALL_SIZE: usize = 136;
pub const DEAD_WALL_SIZE: usize = 14;
pub const INITIAL_DEAL_SIZE: usize = 53;
pub const LIVE_WALL_AFTER_SPLIT: usize = WALL_SIZE - DEAD_WALL_SIZE;
pub const LIVE_WALL_AFTER_DEAL: usize = LIVE_WALL_AFTER_SPLIT - INITIAL_DEAL_SIZE;

/// Shuffled wall with live and dead sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wall {
    layout: WallLayout,
}

/// Result of dealing from a [`Wall`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DealResult {
    pub hands: [Hand; 4],
    pub dealer: usize,
    pub live_remaining: usize,
}

impl Wall {
    pub fn new(rules: &RulesConfig, mut rng: impl Rng) -> Self {
        let mut tiles = standard_set(rules.aka_dora);
        tiles.shuffle(&mut rng);

        let dead = tiles.split_off(tiles.len() - DEAD_WALL_SIZE);
        let layout = WallLayout::new(tiles, dead).expect("standard wall layout is valid");

        Self { layout }
    }

    pub fn from_parts(live: Vec<Tile>, dead: Vec<Tile>) -> Result<Self, Error> {
        Ok(Self {
            layout: WallLayout::new(live, dead)?,
        })
    }

    pub fn layout(&self) -> &WallLayout {
        &self.layout
    }

    pub fn live_remaining(&self) -> usize {
        self.layout.live().len()
    }

    pub fn dead_wall(&self) -> &[Tile] {
        self.layout.dead()
    }

    pub fn dora_indicator(&self) -> Tile {
        self.layout.dora_indicator()
    }

    pub fn dora_indicators(&self) -> Vec<Tile> {
        self.layout.dora_indicators()
    }

    pub fn ura_dora_indicators(&self) -> Vec<Tile> {
        self.layout.ura_dora_indicators()
    }

    pub fn live_drawn(&self) -> usize {
        self.layout.live_drawn()
    }

    /// Deals 13 tiles to each seat and a 14th to the dealer.
    pub fn deal(&mut self, dealer: usize) -> Result<DealResult, Error> {
        if dealer >= 4 {
            return Err(Error::InvalidSeat(dealer));
        }

        let mut hands = [Hand::empty(), Hand::empty(), Hand::empty(), Hand::empty()];

        for _ in 0..3 {
            for offset in 0..4 {
                let seat = (dealer + offset) % 4;
                for _ in 0..4 {
                    let tile = self.layout.draw_live()?;
                    hands[seat].concealed_mut().push(tile);
                }
            }
        }

        for offset in 0..4 {
            let seat = (dealer + offset) % 4;
            let tile = self.layout.draw_live()?;
            hands[seat].concealed_mut().push(tile);
        }

        let tile = self.layout.draw_live()?;
        hands[dealer].concealed_mut().push(tile);

        for hand in &mut hands {
            hand.sort_concealed();
        }

        validate_deal_counts(&hands, dealer)?;

        Ok(DealResult {
            hands,
            dealer,
            live_remaining: self.live_remaining(),
        })
    }

    pub fn draw_live(&mut self) -> Result<Tile, Error> {
        self.layout.draw_live()
    }

    pub fn apply_kan(&mut self) -> Result<(Tile, Tile), Error> {
        self.layout.apply_kan()
    }

    pub const fn kan_count(&self) -> u8 {
        self.layout.kan_count()
    }
}

impl DealResult {
    pub fn hand(&self, seat: usize) -> &Hand {
        &self.hands[seat]
    }

    pub fn dealer_hand_size(&self) -> usize {
        DEALER_HAND_SIZE
    }

    pub fn non_dealer_hand_size(&self) -> usize {
        NON_DEALER_HAND_SIZE
    }
}
