use crate::Error;
use crate::tile::Tile;
use crate::wall::{DEAD_WALL_SIZE, WALL_SIZE};

/// Live and dead wall partitions after the initial shuffle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallLayout {
    live: Vec<Tile>,
    dead: Vec<Tile>,
}

impl WallLayout {
    pub fn new(live: Vec<Tile>, dead: Vec<Tile>) -> Result<Self, Error> {
        let layout = Self { live, dead };
        layout.validate()?;
        Ok(layout)
    }

    pub fn live(&self) -> &[Tile] {
        &self.live
    }

    pub fn dead(&self) -> &[Tile] {
        &self.dead
    }

    pub fn live_drawn(&self) -> usize {
        (WALL_SIZE - DEAD_WALL_SIZE) - self.live.len()
    }

    pub fn dora_indicator(&self) -> Tile {
        self.dead[0]
    }

    pub fn draw_live(&mut self) -> Result<Tile, Error> {
        if self.live.is_empty() {
            return Err(Error::LiveWallExhausted);
        }
        Ok(self.live.remove(0))
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
