//! Scrollable riichi reference text — presentation mirror of `docs/RULES.md`.

pub const LEGEND: &str = "Legend: closed/open han  [C] menzen only  [−] open −1 han  [*] yakuman";

pub const SECTIONS: &[(&str, &[&str])] = &[
    (
        "STANDARD PROFILE",
        &[
            "136 tiles (4×34); red fives when aka_dora enabled.",
            "25,000 starting points; hanchan or east-only match length.",
            "Kiriage off by default — when on, fu rounds up before limits.",
        ],
    ),
    (
        "WINNING",
        &[
            "Standard form: four melds + one pair (14 tiles).",
            "Chiitoitsu: seven pairs (menzen only).",
            "Tsumo — self-draw after drawing on your turn.",
            "Ron — win on another player's discard in reaction phase.",
            "Tenpai — 13 tiles one tile from completion.",
            "Open hand: any chi, pon, open kan, or added kan meld.",
            "Closed kan and concealed tiles count as menzen.",
        ],
    ),
    (
        "CALLS",
        &[
            "Priority: ron > pon / open kan > chi.",
            "Chi — kamicha only (seat after discarder); left/mid/right ok.",
            "Pon — any seat; beats chi.",
            "Open kan (daiminkan) — any seat; beats chi.",
            "Closed kan (ankan) — on your discard turn; dora + rinshan draw.",
            "Added kan (kakan) — upgrade pon on discard turn; dora, rinshan,",
            "  then discard; others may chankan (ron on added tile).",
            "Any kan reveals next dora and voids ippatsu for all riichi.",
            "Double / triple ron when enabled in config (default: double on).",
        ],
    ),
    (
        "RIICHI & TIMING YAKU",
        &[
            "Riichi            1/1  [C]  1000 stick; menzen tenpai discard",
            "Double riichi     2/2  [C]  first discard in seat, no prior calls",
            "Ippatsu           1/1  [C]  riichi win with no call/kan between",
            "Menzen tsumo      1/—  [C]  closed tsumo (not with tenhou)",
            "Haitei raoyue     1/1       tsumo on last live-wall tile",
            "Houtei raoyui     1/1       ron when live wall is empty",
            "Rinshan kaihou    1/1       tsumo on rinshan replacement",
            "Chankan           1/1       ron on kakan tile",
            "Renhou            5/—  [C]  kamicha ron on dealer's first discard",
            "Tenhou / Chiihou 13/—  [*]  dealer / non-dealer first-turn tsumo",
            "Timing yaku skipped when tenhou or chiihou applies.",
        ],
    ),
    (
        "PATTERN YAKU",
        &[
            "Tanyao            1/1       all simples (2–8); honors ok open",
            "Pinfu             1/—  [C]  four sequences, ryanmen non-yakuhai wait",
            "Yakuhai           1/1       seat wind, round wind, or dragon triplet",
            "Chiitoitsu        2/—  [C]  seven pairs; loses to standard patterns",
            "Toitoi            2/2       all triplets",
            "Iipeikou          1/—  [C]  one identical closed sequence pair",
            "Ryanpeikou        3/—  [C]  two identical closed sequence pairs",
            "Sanshoku doujun   2/1  [−]  same numbered sequence in all suits",
            "Ittsu             2/1  [−]  123-456-789 in one suit",
            "Honitsu           3/2  [−]  one suit plus honors",
            "Chinitsu          6/5  [−]  one suit only",
            "Chanta            2/1  [−]  every set + pair has terminal or honor",
            "Junchan           3/2  [−]  every set + pair has 1 or 9 (no honors)",
        ],
    ),
    (
        "YAKUMAN [*]",
        &[
            "Kokushi musou     13 han  thirteen orphans + one duplicate",
            "Suuankou          13 han  four concealed triplets",
            "Daisangen         13 han  triplets of all three dragons",
            "Shousuushii       13 han  three wind triplets + wind pair",
            "Daisuushii        26 han  four wind triplets",
            "Chuuren poutou    13 han  nine gates (one suit)",
            "Ryuuiisou         13 han  all green tiles",
            "Suukantsu         13 han  four kans",
            "Specific yakuman kept over generic (e.g. daisuushii > shousuushii).",
            "Yakuman hands use fixed 30 fu for limit calculation.",
        ],
    ),
    (
        "DORA (EXTRA HAN)",
        &[
            "Dora — always; one indicator at deal + one per kan.",
            "Ura dora — riichi wins; indicators under each dora indicator.",
            "Aka dora — when enabled; one han per red five in winning hand.",
            "Indicator advances: 9→1 in suit; E→S→W→N→E; wd→gd→rd→wd.",
            "Red fives count as their suit's five for dora matching.",
        ],
    ),
    (
        "FU CALCULATION",
        &[
            "1. Yakuman — fixed 30 fu; stop.",
            "2. Chiitoitsu — fixed 25 fu; kiriage; stop.",
            "3. Pinfu — 20 fu tsumo, 30 fu ron; stop (no tsumo +2).",
            "4. Base — 20 fu.",
            "5. Open melds — chi 0; open pon simple +2 / term-honor +4;",
            "     open or added kan = 4× open triplet fu.",
            "6. Concealed melds — closed simple triplet +4; term-honor +8.",
            "7. Pair — seat wind, round wind, or dragon pair +2 each.",
            "8. Wait — tanki / kanchan / penchan +2; ryanmen +0.",
            "9. Win type — closed ron +10; tsumo +2 (except pinfu).",
            "10. Kiriage — if enabled, round up to nearest 10.",
            "11. Minimum — open hand or any ron: at least 30 fu.",
            "Triplet fu: simple open 2 / closed 4; term-honor open 4 / closed 8.",
            "Kan fu = 4× corresponding open triplet fu.",
        ],
    ),
    (
        "HAN & LIMIT BANDS",
        &[
            "total_han = yaku han (with open −1) + dora + ura + aka.",
            "Normal basic = fu × 2^(han + 2).",
            "Mangan     2000 basic — han≥5, or 4 han 40+ fu, or 3 han 70+ fu",
            "Haneman    3000 basic — han ≥ 6",
            "Baiman     4000 basic — han ≥ 8",
            "Sanbaiman  6000 basic — han ≥ 11",
            "Yakuman    8000 basic — han ≥ 13 (daisuushii 26 han same band)",
        ],
    ),
    (
        "PAYMENTS",
        &[
            "All transfers rounded up to nearest 100.",
            "Ron — ko winner: discarder pays basic×4 + honba×300 + riichi sticks.",
            "Ron — oya winner: discarder pays basic×6 + honba×300 + sticks.",
            "Tsumo — ko winner: each child basic, dealer basic×2; +honba×100 each.",
            "Tsumo — oya winner: each opponent basic×2; +honba×100 each.",
            "Riichi sticks go to winner after collection.",
            "Examples (no honba, no sticks):",
            "  1 han 30 fu — ko ron 1000, oya ron 1500, ko tsumo 1000 total",
            "  2 han 30 fu — ko ron 2000, oya ron 2900, ko tsumo 2000 total",
            "  Mangan — ko ron 8000, oya ron 12000, ko tsumo 8000 total",
            "  Haneman — ko ron 12000, oya ron 18000",
            "  Yakuman — ko ron 32000, oya ron 48000",
        ],
    ),
    (
        "RIICHI",
        &[
            "Menzen, tenpai, and at least 1000 points required.",
            "Costs 1000 (table stick); declare on tenpai-preserving discard.",
            "Double riichi — first discard in seat, no calls before declaration.",
            "Ippatsu — +1 han if no call or kan between riichi and win.",
        ],
    ),
    (
        "FURITEN",
        &[
            "Furiten player cannot ron.",
            "Discard furiten — winning tile in your discard pool; never clears.",
            "Temporary furiten — could ron but passed; clears on next draw.",
            "Riichi furiten — riichi player passed on winning discard; permanent.",
        ],
    ),
    (
        "EXHAUSTIVE DRAW",
        &[
            "Live wall empty after a discard.",
            "Tenpai players gain 3000 ÷ (tenpai count) each.",
            "Noten players pay 3000 ÷ (noten count) each.",
            "All tenpai or all noten — no payments.",
        ],
    ),
    (
        "MATCH FLOW",
        &[
            "Honba +1 when dealer wins, dealer tenpai at exhaustive draw,",
            "  or at four-kongs / four-riichis abortive; resets on seat advance.",
            "Renchan — dealer keeps seat on win, dealer tenpai at exhaustive",
            "  or four-kongs / four-riichis abortive draw.",
            "Dealer advances after non-dealer win or draw without dealer tenpai.",
            "Match ends after South 4 (hanchan) or East 4 (east-only),",
            "  or when any seat reaches target_score if set.",
        ],
    ),
    (
        "ABORTIVE DRAWS",
        &[
            "Nine terminals — dealer first turn, ≥9 distinct term/honor types.",
            "Four winds — all four first discards same wind tile.",
            "Four kongs — fourth kan declared.",
            "Four riichis — fourth riichi declaration.",
            "Nine terminals / four winds — dealer and honba unchanged.",
            "Four kongs / four riichis — exhaustive-draw rotation rules.",
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
