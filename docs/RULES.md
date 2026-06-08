# Standard riichi rules (v0)

This document describes the **standard** rules profile implemented in `librrmj/src/rules/standard/`. All yaku, fu, and payment logic lives under that module and is reached only through `RulesProfile` / `RulesRegistry`.

## Profile

| Item | Value |
|------|-------|
| Profile ID | `RulesProfileId::Standard` |
| Tiles | 136 (4×34, including red fives when `aka_dora` is enabled) |
| Starting points | 25,000 per player |
| Rounds | Single-hand scoring only in v0 (match flow is Phase 5) |

## Winning

A hand wins with either:

- **Standard form**: four melds (triplets or sequences) plus one pair (14 tiles total).
- **Chiitoitsu**: seven pairs (menzen only).

Win types:

- **Tsumo** — self-draw on your turn after drawing.
- **Ron** — winning on another player's discard during the reaction phase.

**Tenpai** — a 13-tile hand that completes with one more tile (used for riichi declaration and exhaustive draw).

**Furiten (v0)** — if you have discarded a tile matching the winning tile's identity, you cannot ron on that tile.

## Yaku (v0)

| Yaku | Han | Notes |
|------|-----|-------|
| Riichi | 1 | Declared with 1,000-point stick |
| Menzen tsumo | 1 | Closed tsumo |
| Tanyao | 1 | All simples (2–8) |
| Pinfu | 1 | Menzen, all sequences, non-yakuhai wait (basic check) |
| Yakuhai | 1 | Seat wind, round wind (East), or dragon triplet |

Dora indicators add extra han; ura dora counts after a riichi win; aka dora counts red fives when enabled.

## Fu and limits (simplified v0)

Fu is rounded up to 10 and combined with han using standard mangan thresholds (5 han, 4 han 40 fu, etc.). Payments are rounded to 100.

## Payments

- **Ron** — discarder pays `base × 4` (plus honba and riichi sticks).
- **Tsumo** — each opponent pays their share (dealer pays 2× child share when applicable), plus honba per player.

## Riichi

- Requires menzen, tenpai, and at least 1,000 points.
- Costs 1,000 (placed as a table stick).
- Declared on a discard; hand enters the reaction phase as usual.

## Exhaustive draw

When the live wall is empty after a discard:

- Tenpai players split 3,000 points from noten players.
- If everyone is tenpai or everyone is noten, no payments.

## Architecture boundary

`state/` dispatches wins and draws through `RulesRegistry::get(profile)` — it does not embed yaku or scoring tables. To change rules, implement a new `RulesProfile` and register it; do not branch on profile ID outside `rules/`.
