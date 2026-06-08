use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Man,
    Pin,
    Sou,
}

impl Suit {
    pub const ALL: [Self; 3] = [Self::Man, Self::Pin, Self::Sou];

    pub const fn as_char(self) -> char {
        match self {
            Self::Man => 'm',
            Self::Pin => 'p',
            Self::Sou => 's',
        }
    }

    pub const fn from_char(ch: char) -> Option<Self> {
        match ch {
            'm' => Some(Self::Man),
            'p' => Some(Self::Pin),
            's' => Some(Self::Sou),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

impl Wind {
    pub const ALL: [Self; 4] = [Self::East, Self::South, Self::West, Self::North];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::East => "E",
            Self::South => "S",
            Self::West => "W",
            Self::North => "N",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dragon {
    White,
    Green,
    Red,
}

impl Dragon {
    pub const ALL: [Self; 3] = [Self::White, Self::Green, Self::Red];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::White => "wd",
            Self::Green => "gd",
            Self::Red => "rd",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "wd" => Some(Self::White),
            "gd" => Some(Self::Green),
            "rd" => Some(Self::Red),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileKind {
    Man(u8),
    Pin(u8),
    Sou(u8),
    Wind(Wind),
    Dragon(Dragon),
}

impl TileKind {
    pub fn suit(self) -> Option<Suit> {
        match self {
            Self::Man(_) => Some(Suit::Man),
            Self::Pin(_) => Some(Suit::Pin),
            Self::Sou(_) => Some(Suit::Sou),
            _ => None,
        }
    }

    pub fn rank(self) -> Option<u8> {
        match self {
            Self::Man(r) | Self::Pin(r) | Self::Sou(r) => Some(r),
            _ => None,
        }
    }

    fn category(self) -> u8 {
        match self {
            Self::Man(_) => 0,
            Self::Pin(_) => 1,
            Self::Sou(_) => 2,
            Self::Wind(_) => 3,
            Self::Dragon(_) => 4,
        }
    }

    fn sub_key(self) -> u8 {
        match self {
            Self::Man(r) | Self::Pin(r) | Self::Sou(r) => r,
            Self::Wind(w) => w as u8,
            Self::Dragon(d) => d as u8,
        }
    }
}

impl PartialOrd for TileKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TileKind {
    fn cmp(&self, other: &Self) -> Ordering {
        self.category()
            .cmp(&other.category())
            .then_with(|| self.sub_key().cmp(&other.sub_key()))
    }
}

impl fmt::Display for TileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Man(r) => write!(f, "{r}m"),
            Self::Pin(r) => write!(f, "{r}p"),
            Self::Sou(r) => write!(f, "{r}s"),
            Self::Wind(w) => f.write_str(w.as_str()),
            Self::Dragon(d) => f.write_str(d.as_str()),
        }
    }
}
