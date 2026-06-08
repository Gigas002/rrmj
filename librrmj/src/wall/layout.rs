use crate::Error;
use crate::tile::Tile;
use crate::wall::{DEAD_WALL_SIZE, WALL_SIZE};

/// Live and dead wall partitions after the initial shuffle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallLayout {
    live: Vec<Tile>,
    dead: Vec<Tile>,
    kan_count: u8,
    rinshan_taken: u8,
}

impl WallLayout {
    pub fn new(live: Vec<Tile>, dead: Vec<Tile>) -> Result<Self, Error> {
        let layout = Self {
            live,
            dead,
            kan_count: 0,
            rinshan_taken: 0,
        };
        layout.validate()?;
        Ok(layout)
    }

    pub fn live(&self) -> &[Tile] {
        &self.live
    }

    pub fn dead(&self) -> &[Tile] {
        &self.dead
    }

    pub const fn kan_count(&self) -> u8 {
        self.kan_count
    }

    pub fn live_drawn(&self) -> usize {
        (WALL_SIZE - DEAD_WALL_SIZE) - self.live.len()
    }

    pub fn dora_indicator(&self) -> Tile {
        self.dead[0]
    }

    pub fn dora_indicators(&self) -> Vec<Tile> {
        let count = 1 + self.kan_count as usize;
        (0..count).map(|index| self.dead[index * 2]).collect()
    }

    pub fn ura_dora_indicators(&self) -> Vec<Tile> {
        let count = 1 + self.kan_count as usize;
        (0..count).map(|index| self.dead[index * 2 + 1]).collect()
    }

    pub fn draw_live(&mut self) -> Result<Tile, Error> {
        if self.live.is_empty() {
            return Err(Error::LiveWallExhausted);
        }
        Ok(self.live.remove(0))
    }

    /// Draw a rinshan tile and reveal the next dora indicator after a kan.
    pub fn apply_kan(&mut self) -> Result<(Tile, Tile), Error> {
        let rinshan_index = 4 + self.rinshan_taken as usize;
        let dora_index = 2 * (self.kan_count as usize + 1);

        if rinshan_index >= DEAD_WALL_SIZE {
            return Err(Error::RinshanExhausted);
        }
        if dora_index >= DEAD_WALL_SIZE {
            return Err(Error::DoraRevealExhausted);
        }

        self.kan_count += 1;
        self.rinshan_taken += 1;
        Ok((self.dead[rinshan_index], self.dead[dora_index]))
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.dead.len() != DEAD_WALL_SIZE {
            return Err(Error::InvalidDeadWallSize {
                expected: DEAD_WALL_SIZE,
                actual: self.dead.len(),
            });
        }

        let total = self.live.len() + self.dead.len();
        if total > WALL_SIZE {
            return Err(Error::InvalidWallSize { actual: total });
        }

        Ok(())
    }
}
