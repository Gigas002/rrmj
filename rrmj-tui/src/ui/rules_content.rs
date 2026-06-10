//! Scrollable riichi reference text (cheatsheet layout).

pub const LEGEND: &str = "Legend: [C] closed only  [+] +1 han if closed  [*] yakuman";

pub const SECTIONS: &[(&str, &[&str])] = &[
    (
        "CLOSED HAND REQUIRED",
        &[
            "Riichi          1 han  [C]  1000 pt bet, cannot change hand",
            "Ippatsu         +1 han [C]  win within one turn of riichi",
            "Menzen tsumo    1 han  [C]  self-draw with closed hand",
            "Pinfu           1 han  [C]  all sequences, non-yakuhai wait",
            "  234 345 456 678 789  22  (two-sided wait)",
            "Iipeikou        1 han  [C]  two identical sequences",
            "  123m 123m 456p 789s  33s",
            "Ryanpeikou      3 han  [C]  two pairs of identical sequences",
            "Chiitoitsu      2 han  [C]  seven different pairs",
        ],
    ),
    (
        "WIN TIMING",
        &[
            "Haitei / Houtei  1 han  win on last draw or last discard",
            "Rinshan kaihou  1 han  win on replacement tile after kan",
            "Chankan         1 han  win on tile added to a pon (kakan)",
            "Renhou          5 han  win on discard before first draw",
            "Tenhou / Chiihou [*]  blessing of heaven / earth",
        ],
    ),
    (
        "COMMON YAKU",
        &[
            "Yakuhai         1 han  triplet of dragons, seat wind, or round wind",
            "Tanyao          1 han  all simples (no 1, 9, or honors)",
            "Sanshoku        2 han  [+] same sequence in all three suits",
            "Ittsu           2 han  [+] pure straight 123-456-789 one suit",
            "Chanta          2 han  [+] terminals/honors in every meld + pair",
            "Junchan         3 han  [+] terminals/honors only (no simples)",
            "Toitoi          2 han  all triplets / quads",
            "Honitsu         3 han  [+] half flush (one suit + honors)",
            "Chinitsu        6 han  [+] full flush (one suit only)",
        ],
    ),
    (
        "YAKUMAN [*]",
        &[
            "Kokushi musou   13 orphans (one of each honor + terminal + one pair)",
            "Suuankou        four concealed triplets",
            "Daisangen       big three dragons",
            "Shousuushii     little four winds",
            "Daisuushii      big four winds",
            "Chuuren poutou  nine gates",
            "Ryuuiisou       all green tiles",
            "Suukantsu       four kans",
        ],
    ),
    (
        "FU CALCULATION",
        &[
            "1. Sum han from yaku + dora.",
            "2. Group fu — triplets / quads:",
            "     simple open 2   closed 4",
            "     honor open 4    closed 8",
            "     kan open 8      closed 16",
            "3. Wait / pair fu:",
            "     edge / closed / single wait  2",
            "     dragon or seat / round wind pair  2",
            "4. Hand fu: base 20; +10 closed ron; +2 tsumo.",
            "5. Round up to nearest 10 (e.g. 32 -> 40).",
            "Exhaustive draw: tenpai split 3000 from noten.",
        ],
    ),
    (
        "DEALER (OYA) POINTS — ron / tsumo (each)",
        &[
            "        30fu   40fu   50fu   60fu   70fu",
            "1 han   1500   2000   2400   2900   3400",
            "2 han   2900   3900   4800   5800   6800",
            "3 han   5800   7700   9600  11600  12000 mangan",
            "4 han  11600  12000 mangan",
            "5 han  12000 mangan",
            "6-7    18000 haneman",
            "8-10   24000 baiman",
            "11-12  36000 sanbaiman",
            "13+    48000 yakuman",
            "Honba: +300 per stick to each payment.",
        ],
    ),
    (
        "NON-DEALER (KO) POINTS — ron / tsumo from each",
        &[
            "        30fu   40fu   50fu   60fu   70fu",
            "1 han   1000   1300   1600   2000   2300",
            "2 han   2000   2600   3200   3900   4500",
            "3 han   3900   5200   6400   7700   8000 mangan",
            "4 han   7700   8000 mangan",
            "5 han   8000 mangan",
            "6-7    12000 haneman",
            "8-10   16000 baiman",
            "11-12  24000 sanbaiman",
            "13+    32000 yakuman",
        ],
    ),
    (
        "TILES",
        &[
            "Manzu (m):  1m 2m 3m 4m 5m 6m 7m 8m 9m",
            "Pinzu (p):  1p 2p 3p 4p 5p 6p 7p 8p 9p",
            "Souzu (s):  1s 2s 3s 4s 5s 6s 7s 8s 9s",
            "Winds:      E  S  W  N     Dragons: wd gd rd",
            "Red fives (aka dora): 5mr 5pr 5sr when enabled",
        ],
    ),
    (
        "SEAT WINDS (view from above)",
        &[
            "              North",
            "         West       East (dealer)",
            "              South",
            "Turn order: East -> South -> West -> North",
        ],
    ),
    (
        "PLAY RULES (rrmj v0 engine)",
        &[
            "Implemented yaku: riichi, menzen tsumo, tanyao, pinfu, yakuhai, chiitoitsu.",
            "Dora / ura dora / aka dora per config.",
            "Calls: chi (kamicha), pon, open kan, closed kan.",
            "Ron > pon/kan > chi. Furiten blocks ron on discarded tile.",
            "Riichi: menzen tenpai, 1000 stick, declare on discard.",
            "Abortive: nine terminals, four winds, four kongs, four riichis.",
            "Match: hanchan or east-only; honba / renchan as standard.",
        ],
    ),
    (
        "FOOTER",
        &["Press ? or y to close.  Arrow / PgUp / PgDn scroll."],
    ),
];

pub fn all_lines() -> Vec<&'static str> {
    let mut out = vec![LEGEND, ""];
    for (title, lines) in SECTIONS {
        out.push(*title);
        out.extend_from_slice(lines);
        out.push("");
    }
    out
}

pub fn line_count() -> usize {
    all_lines().len()
}
