use crate::Error;
use crate::tile::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MeldKind {
    Chi,
    Pon,
    OpenKan,
    ClosedKan,
    AddedKan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeldTileCount {
    Three,
    Four,
}

impl MeldKind {
    pub const fn tile_count(self) -> MeldTileCount {
        match self {
            Self::Chi | Self::Pon => MeldTileCount::Three,
            Self::OpenKan | Self::ClosedKan | Self::AddedKan => MeldTileCount::Four,
        }
    }
}

/// A called or declared meld on the table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meld {
    kind: MeldKind,
    tiles: Vec<Tile>,
    /// Tile taken from another player's discard, if any.
    called: Option<Tile>,
}

impl Meld {
    pub fn try_new(kind: MeldKind, tiles: Vec<Tile>, called: Option<Tile>) -> Result<Self, Error> {
        let meld = Self {
            kind,
            tiles,
            called,
        };
        meld.validate()?;
        Ok(meld)
    }

    pub fn chi(tiles: [Tile; 3], called: Tile) -> Result<Self, Error> {
        Self::try_new(MeldKind::Chi, tiles.to_vec(), Some(called))
    }

    pub fn pon(tiles: [Tile; 3], called: Tile) -> Result<Self, Error> {
        Self::try_new(MeldKind::Pon, tiles.to_vec(), Some(called))
    }

    pub fn open_kan(tiles: [Tile; 4], called: Tile) -> Result<Self, Error> {
        Self::try_new(MeldKind::OpenKan, tiles.to_vec(), Some(called))
    }

    pub fn closed_kan(tiles: [Tile; 4]) -> Result<Self, Error> {
        Self::try_new(MeldKind::ClosedKan, tiles.to_vec(), None)
    }

    pub fn added_kan(tile: Tile) -> Result<Self, Error> {
        Self::try_new(MeldKind::AddedKan, vec![tile], None)
    }

    pub const fn kind(&self) -> MeldKind {
        self.kind
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub const fn called(&self) -> Option<Tile> {
        self.called
    }

    pub fn validate(&self) -> Result<(), Error> {
        let expected = match self.kind.tile_count() {
            MeldTileCount::Three => 3,
            MeldTileCount::Four => match self.kind {
                MeldKind::AddedKan => 1,
                _ => 4,
            },
        };

        if self.tiles.len() != expected {
            return Err(Error::InvalidMeldTileCount {
                kind: self.kind,
                expected,
                actual: self.tiles.len(),
            });
        }

        match self.kind {
            MeldKind::Chi | MeldKind::Pon | MeldKind::OpenKan if self.called.is_none() => {
                Err(Error::MissingCalledTile { kind: self.kind })
            }
            MeldKind::ClosedKan | MeldKind::AddedKan if self.called.is_some() => {
                Err(Error::UnexpectedCalledTile { kind: self.kind })
            }
            _ => Ok(()),
        }
    }
}
