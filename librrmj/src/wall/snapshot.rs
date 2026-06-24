use crate::Error;
use crate::tile::Tile;

use super::layout::WallLayout;
use super::{DEAD_WALL_SIZE, Wall};

/// Serializable wall partition for match recordings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WallSnapshot {
    pub live: Vec<Tile>,
    pub dead: Vec<Tile>,
    pub kan_count: u8,
    pub rinshan_taken: u8,
}

impl Wall {
    pub fn snapshot(&self) -> WallSnapshot {
        let layout = self.layout();
        WallSnapshot {
            live: layout.live().to_vec(),
            dead: layout.dead().to_vec(),
            kan_count: layout.kan_count(),
            rinshan_taken: layout.rinshan_taken(),
        }
    }

    pub fn from_snapshot(snapshot: WallSnapshot) -> Result<Self, Error> {
        Ok(Self {
            layout: WallLayout::from_snapshot(snapshot)?,
        })
    }
}

impl WallLayout {
    pub fn snapshot(&self) -> WallSnapshot {
        WallSnapshot {
            live: self.live().to_vec(),
            dead: self.dead().to_vec(),
            kan_count: self.kan_count(),
            rinshan_taken: self.rinshan_taken(),
        }
    }

    pub fn from_snapshot(snapshot: WallSnapshot) -> Result<Self, Error> {
        let layout = Self::from_parts_with_state(
            snapshot.live,
            snapshot.dead,
            snapshot.kan_count,
            snapshot.rinshan_taken,
        )?;
        if layout.dead().len() != DEAD_WALL_SIZE {
            return Err(Error::InvalidDeadWallSize {
                expected: DEAD_WALL_SIZE,
                actual: layout.dead().len(),
            });
        }
        Ok(layout)
    }
}
