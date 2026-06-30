use super::{CheatLine, SectionTone};

pub(crate) fn scoring_lines() -> Vec<CheatLine> {
    let out = vec![
        CheatLine::section(SectionTone::Scoring, "SCORING — HOW TO CALCULATE"),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "1. HAN"),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Sum yaku han (open −1 where marked [−]) + dora + ura + aka.",
        ),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "2. GROUP FU (MELDS)"),
        CheatLine::table_head(
            SectionTone::Scoring,
            "        │ Seq │ Simple triplet │ Term/honor triplet",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " Open   │  0  │       2        │        4",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " Closed │  0  │       4        │        8",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " Kan fu = 4× open triplet fu for that tile",
        ),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "3. WAIT & PAIR FU"),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Tanki / kanchan / penchan wait +2; ryanmen +0.",
        ),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Valued pair (seat wind, round wind, dragon) +2 each.",
        ),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "4. HAND FU"),
        CheatLine::prose(Some(SectionTone::Scoring), "Base 20 fu."),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Closed ron +10; tsumo +2 (except pinfu). Minimum 30 fu on any ron or open hand.",
        ),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Fixed: yakuman 30; chiitoitsu 25; pinfu 20 tsumo / 30 ron.",
        ),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "5. EXHAUSTIVE DRAW"),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Tenpai: gain 3000 ÷ tenpai count. Noten: pay 3000 ÷ noten count.",
        ),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "6. ROUND FU & LIMITS"),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Kiriage (if on): round fu up to next 10. Basic = fu × 2^(han+2) unless limit.",
        ),
        CheatLine::table_head(
            SectionTone::Scoring,
            " Mangan 2000 │ Haneman 3000 │ Baiman 4000 │ Sanbaiman 6000 │ Yakuman 8000",
        ),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Mangan at han≥5, or 4 han 40+ fu, or 3 han 70+ fu.",
        ),
        CheatLine::blank(),
        CheatLine::section(SectionTone::Scoring, "POINT TABLE — DEALER (OYA) RON"),
        CheatLine::table_head(
            SectionTone::Scoring,
            "      │ 30fu │ 40fu │ 50fu │ 60fu │ 70fu+",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 1han │ 1500 │ 2000 │ 2400 │ 2900 │ 3900",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 2han │ 2900 │ 3900 │ 4800 │ 5800 │ Mangan",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 3han │ 5800 │ 7700 │ 9600 │ Mangan │ Mangan",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 4han │ 11600│ Mangan│ Mangan│ Mangan│ Mangan",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 5+han│ Mangan → Haneman → Baiman → Sanbaiman → Yakuman",
        ),
        CheatLine::blank(),
        CheatLine::section(SectionTone::Scoring, "POINT TABLE — NON-DEALER (KO) RON"),
        CheatLine::table_head(
            SectionTone::Scoring,
            "      │ 30fu │ 40fu │ 50fu │ 60fu │ 70fu+",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 1han │ 1000 │ 1300 │ 1600 │ 2000 │ 2600",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 2han │ 2000 │ 2600 │ 3200 │ 3900 │ Mangan",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 3han │ 3900 │ 5200 │ 6400 │ 7700 │ Mangan",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 4han │ 7700 │ Mangan│ Mangan│ Mangan│ Mangan",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " 5+han│ Mangan → Haneman → Baiman → Sanbaiman → Yakuman",
        ),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "TSUMO PAYMENTS (no honba)"),
        CheatLine::table_row(
            SectionTone::Scoring,
            " Ko winner: each opponent pays basic (dealer pays 2× basic)",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " Oya winner: each child pays 2× basic",
        ),
        CheatLine::table_row(
            SectionTone::Scoring,
            " ex 2han 30fu ko tsumo: 2000 total (500+500+1000)",
        ),
        CheatLine::blank(),
        CheatLine::subsection(SectionTone::Scoring, "HONBA & STICKS"),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Ron: +300 × honba from discarder. Tsumo: +100 × honba from each opponent.",
        ),
        CheatLine::prose(
            Some(SectionTone::Scoring),
            "Riichi sticks collected by winner. All payments round up to 100.",
        ),
    ];
    out
}
