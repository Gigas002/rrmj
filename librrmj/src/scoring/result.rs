#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinType {
    Tsumo,
    Ron { from: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Yaku {
    Riichi,
    MenzenTsumo,
    Tanyao,
    Pinfu,
    Yakuhai,
    Dora,
    UraDora,
    AkaDora,
}

impl Yaku {
    pub const fn han(self) -> u8 {
        match self {
            Self::Riichi | Self::MenzenTsumo | Self::Tanyao | Self::Pinfu | Self::Yakuhai => 1,
            Self::Dora | Self::UraDora | Self::AkaDora => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payment {
    pub payer: usize,
    pub payee: usize,
    pub points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreDelta {
    pub seat: usize,
    pub delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoringResult {
    pub winner: usize,
    pub win_type: WinType,
    pub yaku: Vec<Yaku>,
    pub dora: u8,
    pub ura_dora: u8,
    pub aka_dora: u8,
    pub han: u8,
    pub fu: u8,
    pub payments: Vec<Payment>,
    pub deltas: [i32; 4],
}
