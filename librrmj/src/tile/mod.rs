mod identity;
mod kind;

#[cfg(test)]
mod tests;

use std::fmt;
use std::str::FromStr;

pub use identity::TileIdentity;
pub use kind::{Dragon, Suit, TileKind, Wind};

/// A single mahjong tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    kind: TileKind,
    /// `true` only for aka (red) five of a numbered suit.
    red: bool,
}

impl Tile {
    pub const fn new(kind: TileKind, red: bool) -> Self {
        Self { kind, red }
    }

    pub const fn kind(self) -> TileKind {
        self.kind
    }

    pub const fn is_red(self) -> bool {
        self.red
    }

    pub const fn man(rank: u8) -> Self {
        Self::numbered(Suit::Man, rank)
    }

    pub const fn pin(rank: u8) -> Self {
        Self::numbered(Suit::Pin, rank)
    }

    pub const fn sou(rank: u8) -> Self {
        Self::numbered(Suit::Sou, rank)
    }

    pub const fn numbered(suit: Suit, rank: u8) -> Self {
        debug_assert!(rank >= 1 && rank <= 9);
        let kind = match suit {
            Suit::Man => TileKind::Man(rank),
            Suit::Pin => TileKind::Pin(rank),
            Suit::Sou => TileKind::Sou(rank),
        };
        Self { kind, red: false }
    }

    pub const fn red_five(suit: Suit) -> Self {
        let kind = match suit {
            Suit::Man => TileKind::Man(5),
            Suit::Pin => TileKind::Pin(5),
            Suit::Sou => TileKind::Sou(5),
        };
        Self { kind, red: true }
    }

    pub const fn wind(wind: Wind) -> Self {
        Self {
            kind: TileKind::Wind(wind),
            red: false,
        }
    }

    pub const fn dragon(dragon: Dragon) -> Self {
        Self {
            kind: TileKind::Dragon(dragon),
            red: false,
        }
    }

    pub fn suit(self) -> Option<Suit> {
        self.kind.suit()
    }

    pub fn rank(self) -> Option<u8> {
        self.kind.rank()
    }

    /// Total ordering for hand display: man → pin → sou → winds → dragons;
    /// normal five before red five within the same suit.
    pub fn cmp_sort(self, other: Self) -> std::cmp::Ordering {
        self.kind
            .cmp(&other.kind)
            .then_with(|| self.red.cmp(&other.red))
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp_sort(*other)
    }
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TileKind::Man(r) | TileKind::Pin(r) | TileKind::Sou(r) => {
                let suit = match self.kind {
                    TileKind::Man(_) => 'm',
                    TileKind::Pin(_) => 'p',
                    TileKind::Sou(_) => 's',
                    _ => unreachable!(),
                };
                write!(f, "{r}{suit}")?;
                if self.red {
                    f.write_str("r")?;
                }
            }
            TileKind::Wind(w) => f.write_str(w.as_str())?,
            TileKind::Dragon(d) => f.write_str(d.as_str())?,
        }
        Ok(())
    }
}

impl FromStr for Tile {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (base, red) = s.strip_suffix('r').map(|b| (b, true)).unwrap_or((s, false));

        if let Some(ch) = base.chars().next()
            && matches!(ch, 'E' | 'S' | 'W' | 'N')
            && base.len() == 1
        {
            let wind = match ch {
                'E' => Wind::East,
                'S' => Wind::South,
                'W' => Wind::West,
                'N' => Wind::North,
                _ => unreachable!(),
            };
            return Ok(Self::wind(wind));
        }

        if let Some(dragon) = Dragon::from_label(base) {
            if red {
                return Err(crate::Error::InvalidTileString(s.to_owned()));
            }
            return Ok(Self::dragon(dragon));
        }

        let mut chars = base.chars();
        let rank_ch = chars
            .next()
            .ok_or_else(|| crate::Error::InvalidTileString(s.to_owned()))?;
        let suit_ch = chars
            .next()
            .ok_or_else(|| crate::Error::InvalidTileString(s.to_owned()))?;
        if chars.next().is_some() {
            return Err(crate::Error::InvalidTileString(s.to_owned()));
        }

        let rank = rank_ch
            .to_digit(10)
            .ok_or_else(|| crate::Error::InvalidTileString(s.to_owned()))? as u8;
        if !(1..=9).contains(&rank) {
            return Err(crate::Error::InvalidTileString(s.to_owned()));
        }

        let suit = Suit::from_char(suit_ch)
            .ok_or_else(|| crate::Error::InvalidTileString(s.to_owned()))?;
        if red && rank != 5 {
            return Err(crate::Error::InvalidTileString(s.to_owned()));
        }

        Ok(if red {
            Self::red_five(suit)
        } else {
            Self::numbered(suit, rank)
        })
    }
}

/// Builds the standard 136-tile set.
pub fn standard_set(aka_dora: bool) -> Vec<Tile> {
    let mut tiles = Vec::with_capacity(136);

    for suit in Suit::ALL {
        for rank in 1..=9 {
            if rank == 5 && aka_dora {
                tiles.push(Tile::red_five(suit));
                for _ in 0..3 {
                    tiles.push(Tile::numbered(suit, 5));
                }
            } else {
                for _ in 0..4 {
                    tiles.push(Tile::numbered(suit, rank));
                }
            }
        }
    }

    for wind in Wind::ALL {
        for _ in 0..4 {
            tiles.push(Tile::wind(wind));
        }
    }

    for dragon in Dragon::ALL {
        for _ in 0..4 {
            tiles.push(Tile::dragon(dragon));
        }
    }

    debug_assert_eq!(tiles.len(), 136);
    tiles
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::Tile;
    use std::str::FromStr;

    impl serde::Serialize for Tile {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_string())
        }
    }

    impl<'de> serde::Deserialize<'de> for Tile {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let value = String::deserialize(deserializer)?;
            Self::from_str(&value).map_err(serde::de::Error::custom)
        }
    }
}
