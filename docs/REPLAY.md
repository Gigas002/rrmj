# Match recording format (`*.rrmj.json`)

Version **1** — lossless save points for in-progress games, finished replays, and dev scenarios.

**Filesystem paths are client-owned** (`rrmj-tui` chooses `recordings_dir`). `librrmj` only serializes to/from readers and writers.

## Top-level fields

| Field | Type | Required | Notes |
| ----- | ---- | -------- | ----- |
| `format_version` | `u32` | yes | Must be `1` |
| `recording_id` | string | no | Client-generated stable id |
| `created_at` / `updated_at` | string (ISO-8601) | no | Client metadata |
| `client_version` | string | no | e.g. `rrmj-tui 0.1.0` |
| `title` / `description` | string | no | Debug menu / UI labels |
| `tags` | `[string]` | no | e.g. `chi`, `tsumo`, `match-flow` |
| `rules_profile` | string | yes | e.g. `standard` |
| `rules_config` | object | yes | Same schema as `RulesConfig` |
| `seed` | `u64` | yes | Match RNG seed |
| `players` | `[PlayerSetup; 4]` | yes | Seat bindings |
| `human_seat` | `usize` | no | `0`–`3` |
| `cpu_step_delay_ms` | `u64` | no | TUI pause between CPU decisions (ms) |
| `turn_timer_ms` | `u64` | no | Discard thinking limit (ms); draw is automatic; `0` = off |
| `response_timer_ms` | `u64` | no | Reaction window limit for calls (ms); `0` = off; alias `reaction_pass_delay_ms` |
| `match_status` | `in_progress` \| `finished` | yes | UI filter: Load game vs Replays |
| `dealer` | `usize` | yes | Current dealer seat |
| `round_wind` | `east` \| `south` | yes | |
| `kyoku` | `u8` | yes | |
| `honba` | `u8` | yes | |
| `scores` | `[i32; 4]` | yes | Cumulative match scores |
| `table_riichi_sticks` | `u8` | yes | Sticks on table between hands |
| `hand_index` | `u32` | yes | Hands dealt so far |
| `match_phase` | `in_hand` \| `ended` | yes | |
| `hand` | `HandSnapshot` | yes | Full tile + flow state |
| `events` | `[Event]` | yes | Complete applied history |
| `event_index` | `usize` | yes | Last applied event index |
| `expected_legal_actions` | `[Action]` | no | CI assertion hook only |

## `HandSnapshot`

Contains everything needed to resume mid-hand, including mid-reaction:

- `dealer`, `current_actor`, `phase` (`draw` / `discard` / `reaction` / `ended`)
- `hands` — four seats, concealed tiles + open melds
- `discards` — rivers per seat
- `wall` — `live`, `dead` (14 tiles), `kan_count`, `rinshan_taken`
- `reaction` — optional `{ discarder, tile, responses }` when in reaction phase
- `scores`, `riichi`, `table_riichi_sticks`, `honba` (hand-local copies)
- `last_draw`, `first_discards`, `is_dealer_first_turn`, `end_reason`

Tile conservation (136 tiles) is validated on load.

## `PlayerSetup`

```json
{
  "slot": "human",
  "display_name": "You",
  "ai": { "difficulty": "medium", "personality_seed": 42 }
}
```

`ai` is present when `slot` is `cpu`.

## API (`librrmj`, feature `serde`)

```rust
MatchRecording::capture(&match, &match_setup, human_seat, cpu_step_delay_ms, turn_timer_ms, response_timer_ms, meta);
recording.restore()?;           // lossless Match
recording.apply_until(index)?;  // replay events for regression
recording.to_json() / from_json();
recording.to_writer(&mut w) / from_reader(&mut r);
```

## Client conventions (`rrmj-tui`)

All recordings live in **`recordings_dir`** (default `$XDG_DATA_HOME/rrmj/recordings/`). Menus filter by **`match_status`** — files are never moved between directories.

| `match_status` | Menu | Action |
| -------------- | ---- | ------ |
| `in_progress` | **Load game** | Restore and resume (seat picker; saved `human_seat` recommended) |
| `finished` | **Replays** | Review / playback (§11.4) |

Autosave writes after each engine step (async). On match end, the client sets `match_status = finished` and rewrites the **same** file.

Config key: `recordings_dir` (legacy alias: `saves_dir`).

## Fixtures

Committed scenarios live in `examples/scenarios/*.json` and are exercised by `librrmj/tests/scenarios.rs` and the Debug menu (Phase 10.1.1).
