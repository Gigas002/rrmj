# Standard riichi rules

This document describes the **standard** rules profile implemented in `librrmj/src/rules/standard/`. All yaku, fu, and payment logic lives under that module and is reached only through `RulesProfile` / `RulesRegistry`.

## Profile

| Item | Value |
|------|-------|
| Profile ID | `RulesProfileId::Standard` |
| Tiles | 136 (4×34; red fives when `aka_dora` is enabled) |
| Starting points | 25,000 per player (configurable) |
| Rounds | Hanchan (East 1–4, South 1–4) or East-only via `MatchLength` |
| Kiriage | Off by default (`RulesConfig::kiriage`); when on, fu is rounded up to the next 10 before limits |

## Winning

A hand wins with either:

- **Standard form** — four melds (triplets or sequences) plus one pair (14 tiles total).
- **Chiitoitsu** — seven pairs (menzen only).

Win types:

- **Tsumo** — self-draw on your turn after drawing.
- **Ron** — winning on another player's discard during the reaction phase.

**Tenpai** — a 13-tile hand that completes with one more tile (used for riichi declaration and exhaustive draw).

**Open vs closed** — a hand with any open meld (chi, pon, open kan, or added kan) is *open*. Closed kan and concealed tiles count as menzen. Several yaku apply an open-hand −1 han penalty (see yaku table).

## Calls

Reaction priority: **ron** > **pon / open kan** > **chi**.

| Call | Rule |
|------|------|
| Chi | Kamicha only (seat immediately after the discarder). Left, middle, and right positions are all legal when tiles allow. |
| Pon | Any seat; takes precedence over chi. |
| Open kan (daiminkan) | Any seat; takes precedence over chi. |
| Closed kan (ankan) | On your own discard turn; reveals dora and draws a rinshan tile. |
| Added kan (kakan) | Upgrade an existing pon on your discard turn; reveals dora, draws rinshan, then discards. Other players may **chankan** (ron on the added tile). |

Any kan (open, closed, or added) reveals the next dora indicator and voids **ippatsu** for all riichi players.

**Double / triple ron** — when `RulesConfig::double_ron` is true (default), two players may win on the same discard; with `triple_ron`, up to three may win. Payments are resolved per winner.

## Yaku

Han values below are **closed / open** where an open penalty applies. A dash means the yaku cannot be scored open (menzen-only). Yaku stack unless noted.

### Riichi and timing

| Yaku | Han (closed / open) | Notes |
|------|---------------------|-------|
| Riichi | 1 / 1 | Declared with 1,000-point stick; menzen tenpai discard |
| Double riichi | 2 / 2 | First discard in seat, no calls before declaration |
| Ippatsu | 1 / 1 | Riichi win with no intervening call or kan |
| Menzen tsumo | 1 / — | Closed tsumo (not awarded with tenhou) |
| Haitei raoyue | 1 / 1 | Tsumo on the last live-wall tile |
| Houtei raoyui | 1 / 1 | Ron when the live wall is empty |
| Rinshan kaihou | 1 / 1 | Tsumo on a rinshan replacement tile |
| Chankan | 1 / 1 | Ron on a tile added during kakan |
| Renhou | 5 / — | Kamicha ron on dealer's first discard before drawing; no calls in the hand |
| Tenhou | 13 / — | Dealer tsumo on the first turn with no discards |
| Chiihou | 13 / — | Non-dealer tsumo on the first draw |

Tenhou and chiihou share one catalog row; chiihou also earns menzen tsumo when applicable. Timing yaku in the haitei / rinshan / chankan / renhou group are not checked when tenhou or chiihou applies.

### Pattern yaku

| Yaku | Han (closed / open) | Notes |
|------|---------------------|-------|
| Tanyao | 1 / 1 | All simples (2–8); honors allowed in open form |
| Pinfu | 1 / — | Menzen, four sequences, pair wait on a non-yakuhai pair, ryanmen wait |
| Yakuhai | 1 / 1 | Triplet (or pair wait completing triplet) of seat wind, round wind (East), or any dragon |
| Chiitoitsu | 2 / — | Seven pairs; superseded by standard-form pattern yaku when both apply |
| Toitoi | 2 / 2 | All triplets |
| Iipeikou | 1 / — | One identical closed sequence pair |
| Ryanpeikou | 3 / — | Two identical closed sequence pairs |
| Sanshoku doujun | 2 / 1 | Same numbered sequence in all three suits |
| Ittsu | 2 / 1 | 1–2–3, 4–5–6, and 7–8–9 in one suit |
| Honitsu | 3 / 2 | One suit plus honors |
| Chinitsu | 6 / 5 | One suit only |
| Chanta | 2 / 1 | Every set and the pair contain a terminal or honor |
| Junchan | 3 / 2 | Every set and the pair contain a 1 or 9 (no honors) |

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

When multiple yakuman apply, more specific hands are kept (e.g. daisuushii over shousuushii and suuankou; daisangen / kokushi / chuuren / ryuuiisou over suuankou). Yakuman hands use a fixed **30 fu** for limit calculation.

### Dora (extra han)

Dora han is added on top of yaku han:

| Source | When counted |
|--------|--------------|
| Dora | Always; one indicator at deal, plus one per kan |
| Ura dora | Riichi wins only; indicators paired under each dora indicator |
| Aka dora | When `aka_dora` is enabled; one han per red five in the winning hand |

Indicator tile → dora tile: numbered tiles advance within suit (9 → 1); winds E→S→W→N→E; dragons white→green→red→white. Red fives count as their suit's five for dora matching.

## Fu calculation

Fu is computed in `rules/standard/fu/` using the minimum-fu winning decomposition unless a fixed-fu exception applies.

1. **Yakuman** — fixed 30 fu; stop.
2. **Chiitoitsu** — fixed 25 fu; apply kiriage (step 8); stop.
3. **Pinfu** — fixed 20 fu on tsumo, 30 fu on ron; stop (no tsumo +2).
4. **Base** — start at 20 fu.
5. **Open melds** — chi 0; open pon simple +2 / terminal-honor +4; open or added kan = 4× open triplet fu; closed kan on an open meld list uses concealed kan rates.
6. **Concealed melds** — from best decomposition: closed simple triplet +4, closed terminal/honor triplet +8.
7. **Pair** — seat wind, round wind (East), or dragon pair +2 each (valued pairs in decomposition).
8. **Wait** — tanki, kanchan, or penchan +2; ryanmen +0; pair wait on the winning tile +2 when applicable.
9. **Win type** — closed ron +10; tsumo +2 (except pinfu, handled in step 3).
10. **Kiriage** — if `config.kiriage`, round up to the nearest 10.
11. **Minimum** — open hand or any ron: fu is at least 30.

**Triplet fu reference** (used in steps 5–6):

| | Simple | Terminal / honor |
|--|--------|------------------|
| Open | 2 | 4 |
| Closed | 4 | 8 |

Kan fu equals four times the corresponding open triplet fu.

## Han total

```
total_han = sum(yaku han, with open −1 where applicable) + dora + ura_dora + aka_dora
```

## Limit hands (mangan bands)

Basic points before ron/tsumo multipliers:

| Condition | Name | Basic points |
|-----------|------|--------------|
| Normal | — | fu × 2^(han + 2) |
| han ≥ 5, or han = 4 and fu ≥ 40, or han = 3 and fu ≥ 70 | Mangan | 2,000 |
| han ≥ 6 | Haneman | 3,000 |
| han ≥ 8 | Baiman | 4,000 |
| han ≥ 11 | Sanbaiman | 6,000 |
| han ≥ 13 | Yakuman | 8,000 |

Daisuushii (26 han) still maps to the yakuman band (8,000 basic).

## Payments

All point transfers are rounded up to the nearest **100**.

### Ron

| Winner | Discarder pays |
|--------|----------------|
| Non-dealer (ko) | basic × 4 + honba × 300 + riichi sticks |
| Dealer (oya) | basic × 6 + honba × 300 + riichi sticks |

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
| Haneman (6 han) | 12,000 | 18,000 | — | — |
| Yakuman (13 han) | 32,000 | 48,000 | — | — |

## Riichi

- Requires menzen, tenpai, and at least 1,000 points.
- Costs 1,000 (placed as a table stick).
- Declared on a discard that leaves tenpai (`is_riichi_discard`).
- **Double riichi** — first discard in seat with no calls before the declaration.
- **Ippatsu** — one extra han if no player calls or declares kan between the riichi declaration and the win.

## Furiten

A player in furiten cannot ron.

| Kind | When set | Cleared |
|------|----------|---------|
| Discard furiten | Winning tile matches a tile in your discard pool | Never for that tile identity |
| Temporary furiten | You could ron but pass | Next draw |
| Riichi furiten | Riichi player passes on a winning discard | Never for the rest of the hand |

## Exhaustive draw

When the live wall is empty after a discard:

- Tenpai players gain 3,000 ÷ (number of tenpai) each.
- Noten players pay 3,000 ÷ (number of noten) each.
- If everyone is tenpai or everyone is noten, no payments.

Tenpai is evaluated per seat through `RulesProfile::is_tenpai`.

## Match flow

- **Honba** — +1 when the dealer wins or is tenpai at an exhaustive draw, or at a four-kongs / four-riichis abortive draw; resets when the dealer seat advances.
- **Renchan** — dealer keeps seat when they win, are tenpai at exhaustive draw, or are tenpai at four-kongs / four-riichis abortive draw.
- **Dealer rotation** — advances after a non-dealer win or exhaustive/abortive draw without dealer tenpai; East 4 → South 1 in hanchan.
- **Match end** — after South 4 (`MatchLength::Hanchan`) or East 4 (`MatchLength::EastOnly`), or when any seat reaches `target_score` if set.

## Abortive draws

Enabled individually in `RulesConfig`:

| Draw | When | Dealer / honba |
|------|------|----------------|
| Nine terminals | Dealer's first turn with ≥9 distinct terminal/honor types in concealed hand | Unchanged |
| Four winds | All four first discards are the same wind tile | Unchanged |
| Four kongs | Fourth kan declared | Exhaustive-draw rotation rules |
| Four riichis | Fourth riichi declaration | Exhaustive-draw rotation rules |

Nine terminals and four winds keep dealer and honba unchanged. Four kongs and four riichis follow exhaustive-draw dealer rotation and honba rules.

## Architecture boundary

`state/` dispatches wins and draws through `RulesRegistry::get(profile)` — it does not embed yaku or scoring tables. To change rules, implement a new `RulesProfile` and register it; do not branch on profile ID outside `rules/`.
