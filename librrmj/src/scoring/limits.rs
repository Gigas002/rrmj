#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LimitBand {
    Normal,
    Mangan,
    Haneman,
    Baiman,
    Sanbaiman,
    Yakuman,
}

fn limit_band(han: u8, fu: u8, kiriage: bool) -> LimitBand {
    if han >= 13 {
        LimitBand::Yakuman
    } else if han >= 11 {
        LimitBand::Sanbaiman
    } else if han >= 8 {
        LimitBand::Baiman
    } else if han >= 6 {
        LimitBand::Haneman
    } else if han >= 5
        || (han == 4 && fu >= 40)
        || (han == 3 && fu >= 70)
        || (kiriage && ((han == 4 && fu >= 30) || (han == 3 && fu >= 60)))
    {
        LimitBand::Mangan
    } else {
        LimitBand::Normal
    }
}

pub fn limit_band_label(han: u8, fu: u8, kiriage: bool) -> &'static str {
    match limit_band(han, fu, kiriage) {
        LimitBand::Normal => "Normal",
        LimitBand::Mangan => "Mangan",
        LimitBand::Haneman => "Haneman",
        LimitBand::Baiman => "Baiman",
        LimitBand::Sanbaiman => "Sanbaiman",
        LimitBand::Yakuman => "Yakuman",
    }
}

/// Basic points; `kiriage` rounds 4 han 30 fu and 3 han 60 fu up to mangan.
pub(crate) fn base_points(han: u8, fu: u8, kiriage: bool) -> i32 {
    match limit_band(han, fu, kiriage) {
        LimitBand::Normal => fu as i32 * 2i32.pow((han + 2) as u32),
        LimitBand::Mangan => 2_000,
        LimitBand::Haneman => 3_000,
        LimitBand::Baiman => 4_000,
        LimitBand::Sanbaiman => 6_000,
        LimitBand::Yakuman => 8_000,
    }
}
