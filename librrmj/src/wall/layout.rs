use crate::Error;
use crate::tile::Tile;
use crate::wall::{DEAD_WALL_SIZE, WALL_SIZE};

/// First rinshan slot in the dead wall. Slots `0..10` hold the dora
/// indicators (even) with their ura indicators (odd); `10..14` are the four
/// rinshan replacement tiles.
pub const RINSHAN_START: usize = 10;

/// Live and dead wall partitions after the initial shuffle.
///
/// After each kan the dead wall gives up a rinshan tile and takes the last
/// tile of the live wall, so it stays at 14 tiles. `live` keeps its tiles; the
/// last `kan_count` of them belong to the dead wall and are never drawn.
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

    pub const fn rinshan_taken(&self) -> u8 {
        self.rinshan_taken
    }

    pub fn live_drawn(&self) -> usize {
        (WALL_SIZE - DEAD_WALL_SIZE) - self.live.len()
    }

    /// Tiles still drawable from the live wall (excludes those moved to the
    /// dead wall by kans).
    pub fn live_remaining(&self) -> usize {
        self.live.len().saturating_sub(self.kan_count as usize)
    }

    /// Every tile still in the live and dead walls (taken rinshan slots excluded).
    pub fn wall_tiles(&self) -> impl Iterator<Item = Tile> + '_ {
        let taken = RINSHAN_START..RINSHAN_START + self.rinshan_taken as usize;
        self.live.iter().copied().chain(
            self.dead
                .iter()
                .enumerate()
                .filter(move |(index, _)| !taken.contains(index))
                .map(|(_, tile)| *tile),
        )
    }

    /// Tiles in the live and dead walls together that no player holds yet.
    pub fn tiles_in_walls(&self) -> usize {
        self.live.len() + self.dead.len() - self.rinshan_taken as usize
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
        if self.live_remaining() == 0 {
            return Err(Error::LiveWallExhausted);
        }
        Ok(self.live.remove(0))
    }

    /// Draw a rinshan tile and reveal the next dora indicator after a kan.
    /// The live wall's last tile moves to the dead wall in exchange.
    pub fn apply_kan(&mut self) -> Result<(Tile, Tile), Error> {
        let rinshan_index = RINSHAN_START + self.rinshan_taken as usize;
        let dora_index = 2 * (self.kan_count as usize + 1);

        if rinshan_index >= DEAD_WALL_SIZE {
            return Err(Error::RinshanExhausted);
        }
        if dora_index >= RINSHAN_START {
            return Err(Error::DoraRevealExhausted);
        }
        if self.live_remaining() == 0 {
            return Err(Error::LiveWallExhausted);
        }

        self.kan_count += 1;
        self.rinshan_taken += 1;
        Ok((self.dead[rinshan_index], self.dead[dora_index]))
    }

    pub fn from_parts_with_state(
        live: Vec<Tile>,
        dead: Vec<Tile>,
        kan_count: u8,
        rinshan_taken: u8,
    ) -> Result<Self, Error> {
        let layout = Self {
            live,
            dead,
            kan_count,
            rinshan_taken,
        };
        layout.validate()?;
        Ok(layout)
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
        let kans = self.kan_count as usize;
        if kans > DEAD_WALL_SIZE - RINSHAN_START {
            return Err(Error::InvalidWall {
                detail: "more than four kans",
            });
        }
        if self.rinshan_taken != self.kan_count {
            return Err(Error::InvalidWall {
                detail: "rinshan_taken must equal kan_count",
            });
        }
        if kans > self.live.len() {
            return Err(Error::InvalidWall {
                detail: "fewer live tiles than kans",
            });
        }

        Ok(())
    }
}
