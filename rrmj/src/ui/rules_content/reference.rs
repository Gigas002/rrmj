use super::{CheatLine, SectionTone};

pub(crate) fn legend_lines() -> Vec<CheatLine> {
    vec![
        CheatLine::legend("Legend: (han) name  [C] menzen only  [−] open −1 han  [*] yakuman"),
        CheatLine::legend("● yaku row   ex: example combination (tile labels match in-game)"),
    ]
}

pub(crate) fn reference_lines() -> Vec<CheatLine> {
    vec![
        CheatLine::section(SectionTone::Reference, "TILES"),
        CheatLine::prose(
            Some(SectionTone::Reference),
            "Manzu (m):  1m 2m 3m 4m 5m 6m 7m 8m 9m",
        ),
        CheatLine::prose(
            Some(SectionTone::Reference),
            "Pinzu (p):  1p 2p 3p 4p 5p 6p 7p 8p 9p",
        ),
        CheatLine::prose(
            Some(SectionTone::Reference),
            "Souzu (s):  1s 2s 3s 4s 5s 6s 7s 8s 9s",
        ),
        CheatLine::prose(
            Some(SectionTone::Reference),
            "Winds: E S W N     Dragons: wd (white) gd (green) rd (red)",
        ),
        CheatLine::prose(
            Some(SectionTone::Reference),
            "Red fives (aka): 5mr 5pr 5sr when aka_dora enabled",
        ),
        CheatLine::blank(),
        CheatLine::section(SectionTone::Reference, "SEAT WINDS (view from above)"),
        CheatLine::prose(Some(SectionTone::Reference), "              North"),
        CheatLine::prose(
            Some(SectionTone::Reference),
            "         West       East (dealer)",
        ),
        CheatLine::prose(Some(SectionTone::Reference), "              South"),
        CheatLine::prose(
            Some(SectionTone::Reference),
            "Turn order: East → South → West → North",
        ),
    ]
}

pub(crate) fn footer_lines() -> Vec<CheatLine> {
    vec![
        CheatLine::blank(),
        CheatLine::footer("Press ? or Esc to close.  ↑↓ PgUp/PgDn scroll."),
        CheatLine::footer("Full rules: docs/RULES.md in the repository."),
    ]
}
