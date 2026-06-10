#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinType {
    Tsumo,
    Ron { from: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Yaku {
    Riichi,
    DoubleRiichi,
    Ippatsu,
    MenzenTsumo,
    Tanyao,
    Pinfu,
    Yakuhai,
    Chiitoitsu,
    Toitoi,
    Iipeikou,
    Ryanpeikou,
    Sanshoku,
    Ittsu,
    Honitsu,
    Chinitsu,
    Chanta,
    Junchan,
    HaiteiHoutei,
    Rinshan,
    Chankan,
    Renhou,
    TenhouChiihou,
    Kokushi,
    Suuankou,
    Daisangen,
    Shousuushii,
    Daisuushii,
    Chuuren,
    Ryuuiisou,
    Suukantsu,
    Dora,
    UraDora,
    AkaDora,
}

impl Yaku {
    /// Closed-hand han before the open −1 penalty on eligible yaku.
    pub const fn closed_han(self) -> u8 {
        match self {
            Self::Riichi
            | Self::Ippatsu
            | Self::MenzenTsumo
            | Self::Tanyao
            | Self::Pinfu
            | Self::Yakuhai
            | Self::Iipeikou => 1,
            Self::DoubleRiichi => 2,
            Self::Chiitoitsu | Self::Toitoi | Self::Sanshoku | Self::Ittsu | Self::Chanta => 2,
            Self::Honitsu | Self::Junchan | Self::Ryanpeikou => 3,
            Self::Chinitsu => 6,
            Self::HaiteiHoutei | Self::Rinshan | Self::Chankan => 1,
            Self::Renhou => 5,
            Self::TenhouChiihou => 13,
            Self::Kokushi
            | Self::Suuankou
            | Self::Daisangen
            | Self::Shousuushii
            | Self::Chuuren
            | Self::Ryuuiisou
            | Self::Suukantsu => 13,
            Self::Daisuushii => 26,
            Self::Dora | Self::UraDora | Self::AkaDora => 0,
        }
    }

    /// Whether this yaku is a limit hand (yakuman band).
    pub const fn is_yakuman(self) -> bool {
        matches!(
            self,
            Self::TenhouChiihou
                | Self::Kokushi
                | Self::Suuankou
                | Self::Daisangen
                | Self::Shousuushii
                | Self::Daisuushii
                | Self::Chuuren
                | Self::Ryuuiisou
                | Self::Suukantsu
        )
    }

    pub const fn han(self) -> u8 {
        self.closed_han()
    }

    pub const fn open_han_penalty(self) -> bool {
        matches!(
            self,
            Self::Sanshoku
                | Self::Ittsu
                | Self::Honitsu
                | Self::Chinitsu
                | Self::Chanta
                | Self::Junchan
        )
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
