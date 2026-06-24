# Debug scenario catalog

Hand-maintained fixtures in `examples/scenarios/*.json` drive:

- **CI** — `librrmj/tests/scenarios.rs` (no `debug-menu` feature required)
- **TUI dev UI** — Main menu → **Debug** when built with `debug-menu` (off in default release builds)

**Source of truth:** JSON only. There is no Rust scenario builder; do not regenerate fixtures from code.

```bash
cargo test -p librrmj --features serde --test scenarios
```

## How to add a scenario

1. Copy an existing `examples/scenarios/*.json` with a similar setup.
2. Edit `hand`, `wall`, `events`, `event_index`, and metadata (`title`, `description`, `tags`).
3. Optional CI checks go under **`assertions`** (not top-level keys):
   - `assertions.expected_legal_actions` — each action must appear in `legal_actions()` after `restore()`
   - `assertions.expected_yaku` — each yaku must appear in `score_win` when the human seat can tsumo/ron
4. Add a row to the table below.
5. Run `cargo test -p librrmj --features serde --test scenarios`.

Wire format: `docs/REPLAY.md` (`format_version` 3).

## Player scenarios vs debug scenarios

| | **Scenarios** menu (14.6) | **Debug** menu (14.7) |
| --- | --- | --- |
| Build | Default `rrmj-tui` | `--features debug-menu` only |
| Directory | `scenarios_dir` in config (user packs) | `examples/scenarios/` (repo) |
| Assertions in UI | Ignored | Ignored |
| CI | No | Yes (`librrmj/tests/scenarios.rs`) |

## Scenarios (50)

| ID | Tags | What to verify |
| --- | --- | --- |
| `dealer_first_turn` | turn, discard | East opens with 14 tiles; discard menu only |
| `draw_after_discard` | turn, draw | South draw phase after all pass |
| `normal_discard` | discard | Same as dealer first turn |
| `pon_reaction` | calls, pon, reaction | Pon + pass legal for South |
| `chi_kamicha` | calls, chi, reaction | Chi legal for kamicha only |
| `chi_left` | calls, chi, reaction | Chi 4p-5p-6p on 6p discard |
| `chi_middle` | calls, chi, reaction | Chi 4p-5p-6p on 5p discard |
| `chi_right` | calls, chi, reaction | Chi 3p-4p-5p on 3p discard |
| `chi_shimocha_illegal` | calls, chi, reaction | West sees pass only (no chi) |
| `open_kan` | calls, kan, dora, reaction | Open kan available |
| `closed_kan` | calls, kan, dora | Closed kan on four 3p |
| `kakan` | calls, kan, dora | Kakan (pon upgrade) in discard phase |
| `rinshan_discard` | calls, kan, rinshan, discard | 15 tiles after open kan |
| `chankan_ron` | calls, kan, ron, scoring, reaction | Ron on kakan tile (chankan yaku) |
| `ron_on_discard` | ron, scoring, reaction | Ron + pass on 2p discard (tanyao) |
| `ron_over_pon` | calls, reaction | Pon beats chi when both claim |
| `double_ron` | ron, scoring, reaction | Two seats can ron the same discard |
| `triple_ron` | ron, scoring, reaction | Three seats can ron (triple ron enabled) |
| `furiten_temporary` | furiten, ron, reaction | Temporary furiten after passing on a win |
| `furiten_cleared` | furiten, ron, reaction, draw | Temporary furiten clears after a draw |
| `furiten_riichi` | furiten, riichi, ron, reaction | Riichi furiten persists after a draw |
| `yakuhai_ron` | ron, scoring, yakuhai | East-wind yakuhai ron setup |
| `yakuhai_red_ron` | ron, scoring, yakuhai | Red-dragon yakuhai ron |
| `non_dealer_tsumo` | tsumo, scoring | South tsumo win (tanyao + menzen) |
| `dealer_tsumo` | tsumo, scoring, hand-end | East tsumo on honba hand |
| `pinfu_tsumo` | tsumo, scoring, pinfu | Menzen pinfu tsumo |
| `pinfu_ron` | ron, scoring, pinfu | Menzen pinfu ron |
| `menzen_tsumo` | tsumo, scoring, menzen | Menzen + tanyao tsumo |
| `tanyao_ron` | ron, scoring, tanyao | Closed tanyao ron |
| `open_tanyao_ron` | ron, scoring, tanyao, calls | Open pon + tanyao ron |
| `chiitoitsu_tsumo` | tsumo, scoring, chiitoitsu | Chiitoitsu + menzen + tanyao |
| `riichi_tsumo` | riichi, tsumo, scoring | Riichi + menzen + tanyao tsumo |
| `mangan_ron` | ron, scoring, hand-end | Five-han ron (chiitoitsu + double riichi + tanyao) |
| `honba_scoring` | scoring, match-flow, ron | Ron with two honba sticks on the table |
| `dora_kan_chain` | dora, kan, calls | Three dora indicators after two kans |
| `ura_dora_riichi` | dora, riichi, tsumo, scoring | Closed riichi win with ura-dora indicators |
| `aka_dora_on` | dora, scoring | Red fives in hand; aka dora enabled |
| `aka_dora_off` | dora, scoring | Same hand; aka dora disabled in rules config |
| `riichi_declare` | riichi, discard | Riichi declaration available |
| `reaction_pass` | reaction | Pass on safe discard |
| `exhaustive_draw` | draw, hand-end | Wall exhausted |
| `exhaustive_draw_mixed` | draw, hand-end, scoring | Mixed tenpai / noten at exhaustive draw |
| `honba_carry` | match-flow, scoring | Honba stick after dealer win |
| `south_round` | match-flow | South 1 after East round |
| `match_finished` | match-flow | Recording with `match_status = finished` |
| `nine_terminals` | abortive, draw | Nine-terminals abortive |
| `four_winds_abortive` | abortive, draw | Four winds abortive draw |
| `four_kongs_abortive` | abortive, draw, kan | Four kongs abortive draw |
| `four_riichis_abortive` | abortive, draw, riichi | Four riichis abortive draw |
| `multi_reaction` | reaction, calls | Chi + pon both legal |

## Winning-hand coverage

Fixtures with `assertions.expected_yaku` are scored on restore in `librrmj/tests/scenarios.rs`. Full yaku coverage is additionally asserted in:

1. **Table-driven unit tests** — `librrmj/src/rules/standard/win_combinations/tests.rs`
2. **Catalog gate** — `every_implemented_cheatsheet_row_has_win_fixture` in `win_combinations/tests.rs`

Representative scenario fixtures by category:

| Category | Scenario fixtures |
| --- | --- |
| Baseline yaku | `riichi_tsumo`, `pinfu_tsumo`, `chiitoitsu_tsumo`, `yakuhai_red_ron`, … |
| Pattern yaku | covered in unit matrix; honitsu/chinitsu etc. via `win_combinations` |
| Timing yaku | `chankan_ron` (chankan); others in unit matrix |
| Limit hands | `mangan_ron` (mangan band); yakuman in unit matrix |

Tests deliberately cover **win types** (yaku combinations), not meld layout variants (e.g. `1s2s3s` vs `2s3s4s`).

## TUI debug menu (dev builds)

Compile-time feature **`debug-menu`** — **not** enabled in default release builds (`rrmj-tui` default features: `ai` only):

```bash
cargo run -p rrmj-tui                              # no Debug menu
cargo run -p rrmj-tui --features debug-menu      # Main menu → Debug
```

- Lists **`examples/scenarios/*.json`** (repo fixtures; does not use `scenarios_dir`)
- `f` cycles tag filter while browsing
- `i` imports a scenario JSON from any path (defaults to `examples/scenarios/`)
- Pick scenario → seat picker → table (`assertions` are not shown or enforced in the UI)

Community / player packs: use the **Scenarios** menu and `scenarios_dir` — see `docs/REPLAY.md`.
