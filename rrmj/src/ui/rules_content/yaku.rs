use super::{CheatLine, SectionTone};

pub(crate) struct YakuDef {
    pub han: &'static str,
    pub name: &'static str,
    pub tags: &'static str,
    pub note: &'static str,
    pub example: &'static str,
}

fn yaku_block(tone: SectionTone, title: &str, entries: &[YakuDef]) -> Vec<CheatLine> {
    let mut lines = vec![CheatLine::section(tone, title), CheatLine::blank()];
    for y in entries {
        lines.push(CheatLine::yaku_head(
            tone,
            format!("● ({}) {}  {}  {}", y.han, y.name, y.tags, y.note),
        ));
        lines.push(CheatLine::example(tone, format!("    ex: {}", y.example)));
    }
    lines.push(CheatLine::blank());
    lines
}

pub(crate) fn yaku_lines() -> Vec<CheatLine> {
    let mut out = Vec::new();

    out.extend(yaku_block(
        SectionTone::Closed,
        "CLOSED HAND REQUIRED",
        &[
            YakuDef {
                han: "1",
                name: "Riichi",
                tags: "[C]",
                note: "Menzen tenpai; 1000 stick on declare",
                example: "closed 13 tiles → discard into tenpai (e.g. 2m3m4m 5p6p7p 3s4s5s 6s7s8s 2p2p + declare)",
            },
            YakuDef {
                han: "1",
                name: "Menzen tsumo",
                tags: "[C]",
                note: "Self-draw win on a closed hand",
                example: "same shapes as below, win by tsumo (not with tenhou)",
            },
            YakuDef {
                han: "1",
                name: "Pinfu",
                tags: "[C]",
                note: "Four sequences; ryanmen wait; non-yakuhai pair",
                example: "2m3m4m 5p6p7p 3s4s5s 8s8s  → wait 2s or 5s",
            },
            YakuDef {
                han: "1",
                name: "Iipeikou",
                tags: "[C]",
                note: "One identical closed sequence pair",
                example: "1m2m3m 1m2m3m 4p5p6p 7s7s",
            },
            YakuDef {
                han: "3",
                name: "Ryanpeikou",
                tags: "[C]",
                note: "Two identical closed sequence pairs",
                example: "2p3p4p 2p3p4p 6m7m8m 6m7m8m",
            },
            YakuDef {
                han: "2",
                name: "Chiitoitsu",
                tags: "[C]",
                note: "Seven pairs (loses to standard pattern yaku)",
                example: "2m2m 3m3m 5p5p 6p6p 7s7s 8s8s 9s9s",
            },
        ],
    ));

    out.extend(yaku_block(
        SectionTone::Timing,
        "RIICHI & WIN TIMING",
        &[
            YakuDef {
                han: "2",
                name: "Double riichi",
                tags: "[C]",
                note: "First discard in seat; no prior calls",
                example: "first turn discard while tenpai (e.g. 1m1m 2m3m4m 5p6p7p 3s4s5s 6s7s8s)",
            },
            YakuDef {
                han: "1",
                name: "Ippatsu",
                tags: "[C]",
                note: "Riichi win with no call/kan between",
                example: "riichi → immediate win before any reaction",
            },
            YakuDef {
                han: "1",
                name: "Haitei raoyue",
                tags: "",
                note: "Tsumo on last live-wall tile",
                example: "draw winning tile when wall empty",
            },
            YakuDef {
                han: "1",
                name: "Houtei raoyui",
                tags: "",
                note: "Ron when live wall is empty",
                example: "ron on final discard after wall exhausted",
            },
            YakuDef {
                han: "1",
                name: "Rinshan kaihou",
                tags: "",
                note: "Tsumo on rinshan after kan",
                example: "kan → rinshan draw completes hand",
            },
            YakuDef {
                han: "1",
                name: "Chankan",
                tags: "",
                note: "Ron on added kan tile",
                example: "opponent kakan 5p → ron with 5p wait",
            },
            YakuDef {
                han: "5",
                name: "Renhou",
                tags: "[C]",
                note: "Kamicha ron on dealer's first discard",
                example: "dealer discards 3s → South ron before drawing",
            },
            YakuDef {
                han: "13",
                name: "Tenhou / Chiihou",
                tags: "[C] [*]",
                note: "Dealer / non-dealer first-turn tsumo",
                example: "deal 14 → tsumo before any discard (chiihou also + menzen tsumo)",
            },
        ],
    ));

    out.extend(yaku_block(
        SectionTone::Pattern,
        "PATTERN — SIMPLES & SEQUENCES",
        &[
            YakuDef {
                han: "1",
                name: "Tanyao",
                tags: "[−]",
                note: "All simples (2–8); honors ok when open",
                example: "2m3m4m 5p6p7p 3s4s5s 6s7s8s 2p2p",
            },
            YakuDef {
                han: "1",
                name: "Yakuhai",
                tags: "",
                note: "Seat wind, round wind, or dragon set",
                example: "rd rd rd 2m3m4m 5p6p7p 3s4s5s 8s8s  (dragon pon + sequences)",
            },
            YakuDef {
                han: "2/1",
                name: "Sanshoku doujun",
                tags: "[−]",
                note: "Same numbered sequence in m, p, s",
                example: "2m3m4m 2p3p4p 2s3s4s 6m7m8m 5p5p",
            },
            YakuDef {
                han: "2/1",
                name: "Ittsu",
                tags: "[−]",
                note: "123-456-789 in one suit",
                example: "1m2m3m 4m5m6m 7m8m9m 3p4p5p 6s6s",
            },
            YakuDef {
                han: "2/1",
                name: "Chanta",
                tags: "[−]",
                note: "Every set + pair has terminal or honor",
                example: "1m2m3m 9p9p9p E E 5s6s7s 1s1s",
            },
            YakuDef {
                han: "3/2",
                name: "Junchan",
                tags: "[−]",
                note: "Every set + pair has 1 or 9 (no honors)",
                example: "1m2m3m 9p9p9p 1s2s3s 7m8m9m 5m5m",
            },
        ],
    ));

    out.extend(yaku_block(
        SectionTone::Pattern,
        "PATTERN — TRIPLETS & FLUSHES",
        &[
            YakuDef {
                han: "2",
                name: "Toitoi",
                tags: "",
                note: "All triplets / quads",
                example: "3m3m3m 5p5p5p 7s7s7s 9s9s9s 2m2m",
            },
            YakuDef {
                han: "3/2",
                name: "Honitsu",
                tags: "[−]",
                note: "One suit plus honors",
                example: "2p3p4p 5p6p7p 8p9p9p 2p2p E E  (pins + east pair)",
            },
            YakuDef {
                han: "6/5",
                name: "Chinitsu",
                tags: "[−]",
                note: "One suit only",
                example: "1s2s3s 4s5s6s 7s8s9s 2s2s 5s5s5s",
            },
        ],
    ));

    out.extend(yaku_block(
        SectionTone::Yakuman,
        "YAKUMAN [*]",
        &[
            YakuDef {
                han: "13",
                name: "Kokushi musou",
                tags: "[C] [*]",
                note: "Thirteen orphans + one duplicate",
                example: "1m9m 1p9p 1s9s E S W N wd gd rd + one duplicate (e.g. 9m)",
            },
            YakuDef {
                han: "13",
                name: "Suuankou",
                tags: "[C] [*]",
                note: "Four concealed triplets",
                example: "2m2m2m 4p4p4p 6s6s6s 8s8s8s 3m3m  (all concealed)",
            },
            YakuDef {
                han: "13",
                name: "Daisangen",
                tags: "[*]",
                note: "Triplets of all three dragons",
                example: "wd wd wd gd gd gd rd rd rd 2m2m  + any pair",
            },
            YakuDef {
                han: "13",
                name: "Shousuushii",
                tags: "[*]",
                note: "Three wind triplets + wind pair",
                example: "E E E S S S W W W N N  + other melds",
            },
            YakuDef {
                han: "26",
                name: "Daisuushii",
                tags: "[*]",
                note: "Four wind triplets",
                example: "E E E S S S W W W N N N  + pair",
            },
            YakuDef {
                han: "13",
                name: "Chuuren poutou",
                tags: "[C] [*]",
                note: "Nine gates in one suit",
                example: "1m1m1m 2m3m4m5m6m7m8m9m9m9m + any man",
            },
            YakuDef {
                han: "13",
                name: "Ryuuiisou",
                tags: "[*]",
                note: "All green tiles",
                example: "2s3s4s 6s6s6s 8s8s8s gd gd gd 2s2s",
            },
            YakuDef {
                han: "13",
                name: "Suukantsu",
                tags: "[*]",
                note: "Four kans declared",
                example: "four kans (open or closed) + winning pair",
            },
        ],
    ));

    out.extend(yaku_block(
        SectionTone::Dora,
        "DORA (EXTRA HAN)",
        &[
            YakuDef {
                han: "+1",
                name: "Dora",
                tags: "",
                note: "Per matching tile vs indicators; +1 per kan reveal",
                example: "indicator 3m → dora 4m; hand with two 4m = +2 han",
            },
            YakuDef {
                han: "+1",
                name: "Ura dora",
                tags: "",
                note: "Riichi wins only; tile under each indicator",
                example: "riichi tsumo/ron flips ura indicators under dora row",
            },
            YakuDef {
                han: "+1",
                name: "Aka dora",
                tags: "",
                note: "When aka_dora enabled; per red five held",
                example: "5pr 5pr in hand = +2 han (red counts as 5p for dora match)",
            },
        ],
    ));

    out.push(CheatLine::prose(
        Some(SectionTone::Dora),
        "Dora order: 1→2…9→1 in suit; E→S→W→N→E; wd→gd→rd→wd",
    ));

    out
}
