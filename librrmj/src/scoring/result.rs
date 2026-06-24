#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
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

    pub const fn label(self) -> &'static str {
        match self {
            Self::Riichi => "Riichi",
            Self::DoubleRiichi => "Double riichi",
            Self::Ippatsu => "Ippatsu",
            Self::MenzenTsumo => "Menzen tsumo",
            Self::Tanyao => "Tanyao",
            Self::Pinfu => "Pinfu",
            Self::Yakuhai => "Yakuhai",
            Self::Chiitoitsu => "Chiitoitsu",
            Self::Toitoi => "Toitoi",
            Self::Iipeikou => "Iipeikou",
            Self::Ryanpeikou => "Ryanpeikou",
            Self::Sanshoku => "Sanshoku doujun",
            Self::Ittsu => "Ittsu",
            Self::Honitsu => "Honitsu",
            Self::Chinitsu => "Chinitsu",
            Self::Chanta => "Chanta",
            Self::Junchan => "Junchan",
            Self::HaiteiHoutei => "Haitei / Houtei",
            Self::Rinshan => "Rinshan kaihou",
            Self::Chankan => "Chankan",
            Self::Renhou => "Renhou",
            Self::TenhouChiihou => "Tenhou / Chiihou",
            Self::Kokushi => "Kokushi musou",
            Self::Suuankou => "Suuankou",
            Self::Daisangen => "Daisangen",
            Self::Shousuushii => "Shousuushii",
            Self::Daisuushii => "Daisuushii",
            Self::Chuuren => "Chuuren poutou",
            Self::Ryuuiisou => "Ryuuiisou",
            Self::Suukantsu => "Suukantsu",
            Self::Dora => "Dora",
            Self::UraDora => "Ura-dora",
            Self::AkaDora => "Aka-dora",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl ScoringResult {
    pub fn win_type_label(&self) -> String {
        match self.win_type {
            WinType::Tsumo => "Tsumo".into(),
            WinType::Ron { from } => format!("Ron (from seat {})", from + 1),
        }
    }

    pub fn limit_label(&self) -> &'static str {
        crate::scoring::limit_band_label(self.han, self.fu)
    }

    pub fn yaku_lines(&self) -> Vec<String> {
        self.yaku
            .iter()
            .map(|yaku| format!("{} ({} han)", yaku.label(), yaku.closed_han()))
            .collect()
    }

    pub fn dora_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.dora > 0 {
            lines.push(format!("Dora: {} han", self.dora));
        }
        if self.ura_dora > 0 {
            lines.push(format!("Ura-dora: {} han", self.ura_dora));
        }
        if self.aka_dora > 0 {
            lines.push(format!("Aka-dora: {} han", self.aka_dora));
        }
        lines
    }

    pub fn payment_lines(&self) -> Vec<String> {
        self.payments
            .iter()
            .map(|payment| {
                format!(
                    "Seat {} → seat {}: {} pts",
                    payment.payer + 1,
                    payment.payee + 1,
                    payment.points
                )
            })
            .collect()
    }

    pub fn delta_lines(&self) -> Vec<String> {
        self.deltas
            .iter()
            .enumerate()
            .filter(|(_, delta)| **delta != 0)
            .map(|(seat, delta)| format!("Seat {}: {:+}", seat + 1, delta))
            .collect()
    }
}
