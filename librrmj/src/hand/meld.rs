use crate::Error;
use crate::tile::Tile;

use super::kan::KanForm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeldKind {
    Chi,
    Pon,
    Kan(KanForm),
}

#[cfg(feature = "serde")]
impl serde::Serialize for MeldKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MeldKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        MeldKind::parse_str(&value).map_err(serde::de::Error::custom)
    }
}

impl MeldKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chi => "Chi",
            Self::Pon => "Pon",
            Self::Kan(KanForm::Open) => "OpenKan",
            Self::Kan(KanForm::Closed) => "ClosedKan",
        }
    }

    fn parse_str(value: &str) -> Result<Self, String> {
        match value {
            "Chi" => Ok(Self::Chi),
            "Pon" => Ok(Self::Pon),
            "OpenKan" | "AddedKan" => Ok(Self::Kan(KanForm::Open)),
            "ClosedKan" => Ok(Self::Kan(KanForm::Closed)),
            other => Err(format!("unknown meld kind: {other}")),
        }
    }

    pub const fn kan_form(self) -> Option<KanForm> {
        match self {
            Self::Kan(form) => Some(form),
            _ => None,
        }
    }

    pub const fn is_open_kan(self) -> bool {
        matches!(self, Self::Kan(KanForm::Open))
    }

    pub const fn is_closed_kan(self) -> bool {
        matches!(self, Self::Kan(KanForm::Closed))
    }
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
            Self::Kan(_) => MeldTileCount::Four,
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
        Self::try_new(MeldKind::Kan(KanForm::Open), tiles.to_vec(), Some(called))
    }

    pub fn closed_kan(tiles: [Tile; 4]) -> Result<Self, Error> {
        Self::try_new(MeldKind::Kan(KanForm::Closed), tiles.to_vec(), None)
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
            MeldTileCount::Four => 4,
        };

        if self.tiles.len() != expected {
            return Err(Error::InvalidMeldTileCount {
                kind: self.kind,
                expected,
                actual: self.tiles.len(),
            });
        }

        match self.kind {
            MeldKind::Chi | MeldKind::Pon | MeldKind::Kan(KanForm::Open) if self.called.is_none() => {
                Err(Error::MissingCalledTile { kind: self.kind })
            }
            MeldKind::Kan(KanForm::Closed) if self.called.is_some() => {
                Err(Error::UnexpectedCalledTile { kind: self.kind })
            }
            _ => Ok(()),
        }
    }
}
