# Standard riichi rules

This document describes the **standard** rules profile implemented in `librrmj/src/rules/standard/`. All yaku, fu, and payment logic lives under that module and is reached only through `RulesProfile` / `RulesRegistry`.

## Profile

| Item | Value |
|------|-------|
| Profile ID | `RulesProfileId::Standard` |
| Tiles | 136 (4×34; red fives when `aka_dora` is enabled) |
| Starting points | 25,000 per player (configurable) |
| Rounds | Hanchan (East 1–4, South 1–4) or East-only via `MatchLength` |
| Kiriage mangan | Off by default (`RulesConfig::kiriage`); when on, 4 han 30 fu and 3 han 60 fu score as mangan |

## Winning

A hand wins with either:

- **Standard form** — four melds (triplets or sequences) plus one pair (14 tiles total).
- **Chiitoitsu** — seven *different* pairs (menzen only; four of a kind is not two pairs).
- **Kokushi musou** — see yakuman.

A complete hand also needs at least one yaku: dora alone is not a yaku, and
neither ron nor tsumo is offered without one.

Red fives are ordinary fives for every shape check (sets, pairs, waits, calls,
furiten); only aka dora counting looks at them.

Win types:

- **Tsumo** — self-draw on your turn after drawing.
- **Ron** — winning on another player's discard during the reaction phase.

**Tenpai** — a 13-tile hand that completes with one more tile (used for riichi declaration and exhaustive draw).

**Open vs closed** — a hand with any open meld (chi, pon, open kan, or added kan) is *open*. Closed kans keep the hand closed (menzen): riichi, menzen tsumo, iipeikou and the closed-ron +10 fu all still apply. Several yaku apply an open-hand −1 han penalty (see yaku table).

## Calls

Reaction priority: **ron** > **pon / open kan** > **chi**.

| Call | Rule |
|------|------|
| Chi | Kamicha only (seat immediately after the discarder). Left, middle, and right positions are all legal when tiles allow. |
| Pon | Any seat; takes precedence over chi. |
| Open kan (daiminkan) | Any seat; takes precedence over chi. |
| Closed kan (ankan) | On your own discard turn; reveals dora and draws a rinshan tile. |
| Added kan (kakan) | Upgrade an existing pon on your discard turn; reveals dora, draws rinshan, then discards. Other players may **chankan** (ron on the added tile). |

Any kan (open, closed, or added) reveals the next dora indicator at once and voids **ippatsu** for all riichi players.

**Kan limits** — a kan needs a tile left in the live wall to replace, and at most four kans are made in a hand. Each kan draws its replacement from the rinshan end of the dead wall and shortens the live wall by one tile, so the dead wall stays at 14. No kan right after a chi or pon (a kan follows a draw).

**No calls on the last discard** — when the live wall is empty, the last discard can only be ronned (houtei).

**Kuikae (swap-calling)** — after a chi or pon you may not discard the called tile, nor (for a chi taken at one end) the tile at the other end of the same run: after chi 3m with 4m-5m, neither 3m nor 6m may be discarded that turn. A call that would leave only such tiles is not offered.

**Double / triple ron** — when `RulesConfig::double_ron` is true (default), two players may win on the same discard; with `triple_ron`, up to three may win. Each winner is paid separately by the discarder; honba and the table's riichi sticks go to the winner closest to the discarder's right (turn order). With `triple_ron` off, three rons on one discard keep the two closest winners (head bump); there is no sanchahou abortive draw. With `double_ron` off, only the closest winner wins.

## Yaku

Han values below are **closed / open** where an open penalty applies. A dash means the yaku cannot be scored open (menzen-only). Yaku stack unless noted.

### Riichi and timing

| Yaku | Han (closed / open) | Notes |
|------|---------------------|-------|
| Riichi | 1 / — | Declared with 1,000-point stick; menzen tenpai discard |
| Double riichi | 2 / — | First discard in seat, no calls before declaration |
| Ippatsu | 1 / — | Riichi win with no intervening call or kan |
| Menzen tsumo | 1 / — | Closed tsumo (not awarded with tenhou) |
| Haitei raoyue | 1 / 1 | Tsumo on the last live-wall tile |
| Houtei raoyui | 1 / 1 | Ron when the live wall is empty |
| Rinshan kaihou | 1 / 1 | Tsumo on a rinshan replacement tile |
| Chankan | 1 / 1 | Ron on a tile added during kakan |
| Renhou | 5 / — | Kamicha ron on dealer's first discard before drawing; no calls in the hand |
| Tenhou | 13 / — | Dealer tsumo on the first turn with no discards |
| Chiihou | 13 / — | Non-dealer tsumo on the first draw, no calls before it |

Tenhou and chiihou share one catalog row. They are yakuman, so no other yaku is listed with them (see yakuman).

### Pattern yaku

| Yaku | Han (closed / open) | Notes |
|------|---------------------|-------|
| Tanyao | 1 / 1 | All simples (2–8); honors allowed in open form |
| Pinfu | 1 / — | Menzen, four sequences, pair wait on a non-yakuhai pair, ryanmen wait |
| Yakuhai | 1 / 1 each | Triplet or kan of a dragon, the seat wind, or the round wind. Counted once per value: two dragon triplets are 2 han, a triplet of a wind that is both seat and round wind (double wind) is 2 han. A pair is never yakuhai |
| Chiitoitsu | 2 / — | Seven different pairs |
| Toitoi | 2 / 2 | All triplets |
| Sanankou | 2 / 2 | Three concealed triplets (closed kans count; a triplet completed by ron does not) |
| Sankantsu | 2 / 2 | Three kans |
| Sanshoku doukou | 2 / 2 | Same numbered triplet in all three suits |
| Shousangen | 2 / 2 | Two dragon triplets and a dragon pair (plus their yakuhai) |
| Honroutou | 2 / 2 | Only terminals and honors (with toitoi or chiitoitsu) |
| Iipeikou | 1 / — | One identical closed sequence pair |
| Ryanpeikou | 3 / — | Two identical closed sequence pairs |
| Sanshoku doujun | 2 / 1 | Same numbered sequence in all three suits |
| Ittsu | 2 / 1 | 1–2–3, 4–5–6, and 7–8–9 in one suit |
| Honitsu | 3 / 2 | One suit plus honors |
| Chinitsu | 6 / 5 | One suit only |
| Chanta | 2 / 1 | Every set and the pair contain a terminal or honor, at least one run |
| Junchan | 3 / 2 | Every set and the pair contain a 1 or 9 (no honors), at least one run |

Open −1 han applies to sanshoku, ittsu, honitsu, chinitsu, chanta, and junchan.

### Yakuman (limit hands)

| Yaku | Han | Notes |
|------|-----|-------|
| Kokushi musou | 13 | Thirteen orphans (one of each terminal/honor plus one duplicate) |
| Suuankou | 13 | Four concealed triplets |
| Daisangen | 13 | Triplets of all three dragons |
| Shousuushii | 13 | Three wind triplets plus a wind pair |
| Daisuushii | 26 | Four wind triplets |
| Chuuren poutou | 13 | Nine gates (one suit, 1–1–1–2–3–4–5–6–7–8–9–9–9 plus one extra) |
| Ryuuiisou | 13 | All green tiles (2/3/4/6/8 sou and green dragon) |
| Suukantsu | 13 | Four kans |
| Tsuuiisou | 13 | Only honor tiles (standard form or seven pairs) |
| Chinroutou | 13 | Only terminals (1 and 9) |

A yakuman hand scores only its yakuman: riichi, ippatsu, dora and the other
regular yaku are not listed. When several yakuman apply, more specific hands
are kept (daisuushii over shousuushii and suuankou; daisangen / kokushi /
chuuren / ryuuiisou over suuankou); the rest stack in the list, but payment is
capped at a single yakuman (see limits). Yakuman hands use a fixed **30 fu**.

**Not supported:** nagashi mangan, counted yakuman (13+ han from regular yaku
is paid as a yakuman, see limits), double yakuman payments, pao
(responsibility payments).

### Choosing the reading

Many complete hands can be read more than one way: a different split into
sets and pair, or a different group the winning tile completed (for example
`2m 3m 4m 2m` waiting on 2m is either a 2m tanki or a 3-4m ryanmen with a 2m
pair). Every reading is scored on its own (yaku and fu together), and the one
paying the most is used; ties go to more han, then more fu. Seven pairs and
kokushi are readings too, so ryanpeikou beats chiitoitsu when it pays more.

### Dora (extra han)

Dora han is added on top of yaku han:

| Source | When counted |
|--------|--------------|
| Dora | Always; one indicator at deal, plus one per kan |
| Ura dora | Riichi wins only; indicators paired under each dora indicator |
| Aka dora | When `aka_dora` is enabled; one han per red five in the winning hand |

Indicator tile → dora tile: numbered tiles advance within suit (9 → 1); winds E→S→W→N→E; dragons white→green→red→white. Red fives count as their suit's five for dora matching.

## Fu calculation

Fu is computed per reading (see *Choosing the reading*) in
`rules/standard/eval/`.

1. **Yakuman** — fixed 30 fu.
2. **Chiitoitsu** — fixed 25 fu (never rounded).
3. **Pinfu** — 20 fu on tsumo, 30 fu on ron (no tsumo +2).
4. **Base** — 20 fu.
5. **Sets** — triplets by the table below; a kan is 4× its triplet. A triplet
   completed by ron (shanpon wait) counts as open.
6. **Pair** — dragon +2, seat wind +2, round wind +2 (a double-wind pair is +4).
7. **Wait** — tanki, kanchan or penchan +2; ryanmen and shanpon +0.
8. **Win type** — closed ron +10; tsumo +2.
9. **Round up** to the next 10.
10. **Minimum** — an open hand scores at least 30 fu (open pinfu shape).

**Triplet fu**:

| | Simple | Terminal / honor |
|--|--------|------------------|
| Open | 2 | 4 |
| Closed | 4 | 8 |

## Han total

```
total_han = sum(yaku han, with open −1 where applicable) + dora + ura_dora + aka_dora
```

## Limit hands (mangan bands)

Basic points before ron/tsumo multipliers:

| Condition | Name | Basic points |
|-----------|------|--------------|
| Normal | — | fu × 2^(han + 2) |
| han ≥ 5, or han = 4 and fu ≥ 40, or han = 3 and fu ≥ 70 (with kiriage: also 4 han 30 fu and 3 han 60 fu) | Mangan | 2,000 |
| han ≥ 6 | Haneman | 3,000 |
| han ≥ 8 | Baiman | 4,000 |
| han ≥ 11 | Sanbaiman | 6,000 |
| han ≥ 13 | Yakuman | 8,000 |

13 han or more is a yakuman (8,000 basic), whether from yakuman or from many
regular han; daisuushii (26 han) and stacked yakuman are still one yakuman.

## Payments

All point transfers are rounded up to the nearest **100**.

### Ron

| Winner | Discarder pays |
|--------|----------------|
| Non-dealer (ko) | basic × 4 + honba × 300 |
| Dealer (oya) | basic × 6 + honba × 300 |

The riichi sticks on the table (1,000 each) go to the winner; the discarder
doesn't pay them.

### Tsumo

Each opponent pays their share plus honba × 100. Riichi sticks go to the winner after collection.

| Winner | Each child pays | Dealer pays |
|--------|-----------------|-------------|
| Non-dealer | basic | basic × 2 |
| Dealer | basic × 2 | — |

### Examples (no honba, no sticks)

| Hand | Ko ron | Oya ron | Ko tsumo (from 3 opponents) | Oya tsumo |
|------|--------|---------|-------------------------------|-----------|
| 1 han 30 fu | 1,000 | 1,500 | 1,000 total | 3,000 total |
| 2 han 30 fu | 2,000 | 2,900 | 2,000 total | 6,000 total |
| Mangan (5 han) | 8,000 | 12,000 | 8,000 total | 12,000 total |
| Haneman (6 han) | 12,000 | 18,000 | 12,000 total | 18,000 total |
| Yakuman (13 han) | 32,000 | 48,000 | 32,000 total | 48,000 total |

`score/tests.rs` checks every cell of the usual table (1–4 han × 20–110 fu
and the limits) against published values.

## Riichi

- Requires a closed hand (closed kans allowed), at least 1,000 points, and at
  least 4 tiles left in the live wall.
- Declared on a discard that leaves the hand tenpai (`is_riichi_discard`).
- Costs 1,000, put on the table with the declaration. If the declaration
  discard is ronned the riichi still stands and the stick goes to the winner
  (standard play would void it; see AUDIT.md §2 follow-ups).
- After riichi the player only discards the drawn tile (tsumogiri), can tsumo
  or ron, and can make a closed kan only of the drawn tile and only when the
  waits stay exactly the same. A player in riichi makes no calls.
- **Double riichi** — first discard in seat with no calls before the declaration.
- **Ippatsu** — one extra han if no player calls or declares kan between the riichi declaration and the win.

## Furiten

A player in furiten cannot ron (tsumo is still allowed). Waits include tiles
that would complete the hand without a yaku.

| Kind | When set | Cleared |
|------|----------|---------|
| Discard furiten | Winning tile matches a tile in your discard pool | Never for that tile identity |
| Temporary furiten | You could ron but pass | Your next draw |
| Riichi furiten | Riichi player passes on a winning discard | Never for the rest of the hand |

## Exhaustive draw

When the live wall is empty after a discard:

- Tenpai players gain 3,000 ÷ (number of tenpai) each.
- Noten players pay 3,000 ÷ (number of noten) each.
- If everyone is tenpai or everyone is noten, no payments.

Tenpai is evaluated per seat through `RulesProfile::is_tenpai`. A hand whose
only wait is a tile it already holds four of (karaten) counts as tenpai.

## Match flow

- **Honba** — +1 after a dealer win and after every draw (exhaustive or
  abortive), whoever deals next; reset to 0 only by a non-dealer win.
- **Renchan** — the dealer keeps the seat after winning, after being tenpai
  at an exhaustive draw, and after any abortive draw.
- **Dealer rotation** — after a non-dealer win or a draw with the dealer
  noten; East 4 → South 1 in a hanchan.
- **Riichi sticks** — stay on the table through draws and go to the next
  winner. Sticks still on the table when the match ends go to first place.
- **Match end** — after the last hand (South 4 for `GameLength::Hanchan`,
  East 4 for `GameLength::EastOnly`); in that hand a dealer repeat plays on
  unless the dealer is already first (agari-yame / tenpai-yame). The match
  also ends as soon as a seat drops below zero (bust) or any seat reaches
  `target_score` if set. There is no west-round extension.
- **Placement** — by score; equal scores rank by seat order (seat 0 dealt
  first).

## Abortive draws

Enabled individually in `RulesConfig`. Every abortive draw keeps the dealer
and adds a honba.

| Draw | When |
|------|------|
| Nine terminals | A player's first turn, before any call or kan, with ≥ 9 different terminals/honors; the player chooses |
| Four winds | All four first discards are the same wind, with no call before |
| Four kongs | The fourth kan of the hand, when the kans belong to two or more players (one player with all four plays on for suukantsu) |
| Four riichis | The fourth player declares riichi |

## Architecture boundary

`state/` dispatches wins and draws through `RulesRegistry::get(profile)` — it does not embed yaku or scoring tables. To change rules, implement a new `RulesProfile` and register it; do not branch on profile ID outside `rules/`.
