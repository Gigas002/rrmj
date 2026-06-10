use crate::tile::Tile;

/// Tiles held face-down (or before exposure).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Concealed {
    tiles: Vec<Tile>,
}

impl Concealed {
    pub fn empty() -> Self {
        Self { tiles: Vec::new() }
    }

    pub fn from_tiles(mut tiles: Vec<Tile>) -> Self {
        tiles.sort();
        Self { tiles }
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub fn push(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }

    pub fn sort(&mut self) {
        self.tiles.sort();
    }

    pub fn contains(&self, tile: Tile) -> bool {
        self.tiles.contains(&tile)
    }

    pub fn remove(&mut self, tile: Tile) -> Result<(), crate::Error> {
        let pos = self
            .tiles
            .iter()
            .position(|t| *t == tile)
            .ok_or(crate::Error::TileNotInHand { tile })?;
        self.tiles.remove(pos);
        Ok(())
    }

    pub fn remove_matching_identity(&mut self, tile: Tile) -> Result<(), crate::Error> {
        let pos = self
            .tiles
            .iter()
            .position(|t| t.matches_identity(tile))
            .ok_or(crate::Error::TileNotInHand { tile })?;
        self.tiles.remove(pos);
        Ok(())
    }
}
