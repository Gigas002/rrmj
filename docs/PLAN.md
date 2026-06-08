# rrmj — Rust architecture + implementation plan

This document is the **human roadmap** and **agent playbook** for **rrmj** (Rust Riichi Mahjong): a **library-first** riichi mahjong rules engine (`librrmj`), a **ratatui** terminal client (`rrmj-tui`), and (post-first-release) a planned **Bevy** graphical client.

**Execution discipline:**

- Library-first crate split, small verifiable phases, strict quality gates (fmt, clippy `-D warnings` with feature matrix, tests, `cargo doc`, `typos`, `cargo deny`).
- **Directory modules** with **sibling `tests.rs`** — tests never live in the same file as logic.
- **Per-integration Cargo features** so minimal installs and CI do not bitrot optional code paths.

**Reference material (to be added as the engine matures):**

- `examples/` — saved game logs, rule presets, AI bench fixtures.
- `docs/RULES.md` — chosen riichi ruleset (yaku list, dora, abortive draws, etc.) as the single source of truth for scoring tests.

---

## 1. Goals and constraints

### 1.1 Goals

- **Correct riichi rules engine**: full 4-player Japanese riichi mahjong in `librrmj` — wall, turns, calls, riichi, dora, wins, scoring, match flow (east round, honba, kyuu/haku).
- **Play vs CPU (first release)**: `rrmj-tui` supports a complete local match against 3 AI opponents at **multiple difficulty tiers**.
- **Online-ready core (designed now, shipped later)**: game progression is **event-sourced** and **deterministic** given a seed + action log; no transport code in `librrmj` until post-release, but the API must not require a rewrite to add multiplayer.
- **Thin clients**: `rrmj-tui` (and later Bevy) are **views + input** only; all legality, state transitions, and scoring live in `librrmj`.
- **Headless testability**: any match can be driven from a sequence of `Action`s or replayed from `Event`s without a UI.
- **Extensible rulesets (designed now, one shipped in v0)**: v0 implements **standard Japanese riichi** only, but rule-specific logic is isolated behind a **`RulesProfile`** boundary so additional rulesets can be added later without rewriting the engine (see §3.5).

### 1.2 Discipline (non-negotiable)

- **Library-first**: **`librrmj`** — tiles, rules, state machine, scoring, AI traits; **`rrmj-tui`** — `main`, ratatui layout, keymap, config/TOML, tracing setup.
- **`rrmj-tui` contains no domain logic** beyond wiring; **`librrmj` does not depend on** `ratatui`, `crossterm`, `clap`, or `toml`, and does not assume a specific logger beyond `tracing`.
- **Determinism**: wall order and AI decisions that need randomness go through an injected **`Rng` / `Seed`** (e.g. `rand_chacha::ChaCha8Rng`) so tests and future netplay replays are reproducible.
- **Step sizing**: small PR-sized phases with explicit **Verify** blocks.
- **Feature matrix in CI**: default, `--all-features`, `--no-default-features` (core must still build: engine + minimal types without AI/TUI extras).
- **Naming**: short, descriptive; prefer clarity over abstraction depth.
- **Code comments**: describe current behavior only (invariants, rule edge cases, non-obvious scoring). No roadmap phase labels or chat context in source.

### 1.3 Non-goals (first release)

- **No** online multiplayer, lobby, or matchmaking in v0.
- **No** Bevy / wgpu graphical client in v0 (architecture note only; separate plan after v0.1).
- **No** additional rulesets beyond **standard** in v0 (no three-player, HK, or custom house-rule packs yet) — but the **`RulesProfile`** architecture in §3.5 must be in place from the first scoring phase so new rulesets are additive, not a rewrite.
- **No** voice, emotes, or account systems.
- **No** full interactive tutorial in v0; TUI provides a **rules / yaku reference** overlay (see §6.5) instead of a guided walkthrough.

### 1.4 Definitions

- **Tile**: one of 34 unique types (man/pin/sou 1–9, winds E/S/W/N, dragons white/green/red), four copies each in the wall.
- **Action**: a player intent (discard, call chi/pon/kan, riichi, tsumo, ron, pass, etc.); may be **illegal** in a given state — only `librrmj` decides.
- **Event**: an **applied** state change (what actually happened); ordered event log is the canonical history for replay and future sync.
- **Seat**: player position (East/South/West/North) relative to the dealer; distinct from **display slot** in UI.
- **Agent**: anything that supplies `Action`s — human (TUI), `Cpu` (AI), or (later) `Remote` via network adapter.
- **Match**: full session (multiple hands, scores, round wind progression) until a win condition or user quit.
- **Rules profile**: a named ruleset implementation (e.g. `standard`) — yaku table, scoring, draw policies, and match-flow hooks. Distinct from **`RulesConfig`**, which tunes parameters within a profile.

---

## 2. Repository layout (target)

Phase 0 establishes this layout; workspace members must match this tree.

```text
rrmj/                          # workspace root
  Cargo.toml                   # members: librrmj, rrmj-tui
  Cargo.lock                   # committed
  deny.toml
  examples/
    rules_default.toml         # (future) rule preset: aka dora, kiriage, etc.
    replays/                   # (future) JSONL event logs for regression
  librrmj/
    Cargo.toml                 # features: ai, serde, test-util, …
    src/
      lib.rs
      error.rs                 # thiserror — rule/phase errors only
      tile/                    # Tile, Suit, tile parsing, sorting
      wall/                    # shuffle, deal, dead wall, dora indicators
      hand/                    # concealed/meld representation, waits
      action/                  # Action enum, legal move generation
      event/                   # Event enum, apply, event log
      state/                   # MatchState, HandState, phase machine
      rules/                   # RulesConfig, RulesProfile trait, registry
        profile/               # trait + shared rule helpers
        standard/              # v0 Japanese riichi (only profile in v0)
      scoring/                 # shared payment types; profile-specific math in rules/*
      game/                    # multi-hand flow, honba, renchan
      agent/                   # Agent trait, PlayerSlot { Human, Cpu, Remote }
      replay/                  # in-memory Replay + event re-apply API
      ai/                      # behind `ai` feature
        easy/
        medium/
        hard/
      rng/                     # seeded RNG wrapper
  rrmj-tui/
    Cargo.toml                 # clap, toml, ratatui, crossterm, tracing-subscriber, librrmj
    src/
      main.rs
      error.rs                 # config/file errors
      app/                     # App state: bridge UI ↔ librrmj
      ui/                      # ratatui widgets (hand, river, status, menu)
      input/                   # keymap, action picker
      config/                  # TOML: difficulty, theme, keybinds
      theme/                   # color palettes / ratatui styles (presentation only)
      cli/
  docs/
    PLAN.md                    # this file
    RULES.md                   # (Phase 4) standard profile: yaku + house rules
    REPLAY.md                  # (post-v0) replay file format spec
    ONLINE.md                  # (post-v0) protocol sketch — stub until then
  .github/workflows/           # build, fmt-clippy, test, doc, typos, deny
```

**Crate boundary rules**

- `librrmj` has **no** `ratatui`, `crossterm`, `clap`, `toml`, or `bevy`; use `tracing` only.
- `rrmj-tui` builds a `MatchConfig` / `AppSettings` from CLI + TOML, then drives `librrmj::Match` — never duplicates rule checks.
- Optional **`rrmj-tui` features** are thin passthroughs to `librrmj` features so packagers can trim optional deps.

**Future crate (not in workspace until post-v0 plan):** `rrmj-bevy` — same boundary as `rrmj-tui`; shares only `librrmj`.

---

## 3. Core data model

### 3.1 Rules configuration

Two layers — do not conflate them:

| Layer            | Role                                                                 |
| ---------------- | -------------------------------------------------------------------- |
| **`RulesProfile`** | Which ruleset (yaku set, scoring model, draw/match policies).      |
| **`RulesConfig`**  | Tunable parameters **within** a profile (aka dora, uma, kiriage).  |

`RulesConfig` (in `librrmj::rules`) centralizes profile-specific knobs for **standard** riichi:

- Starting points (default 25 000), uma, placement bonus.
- Red fives (aka dora) on/off.
- Riichi stick (1 000), honba stick (300).
- Allowed yaku subset and fu rounding (kiriage / round-up table).
- Abortive draw types enabled (nine terminals, four winds, four kongs, four riichis).
- Triple-ron / triple-wind draw policy.

Deserialize from TOML in **`rrmj-tui` only**; `librrmj` exposes `RulesProfileId::Standard` + `RulesConfig::default_for(profile)` for tests.

Every `Match` stores **`(RulesProfileId, RulesConfig)`** at creation; replay files embed both (see §3.3, §11.4).

### 3.2 State machine phases

`HandPhase` (illustrative — refine in implementation):

| Phase            | Description                                      |
| ---------------- | ------------------------------------------------ |
| `Deal`           | Wall built, hands dealt, dora revealed           |
| `Turn`           | Active seat may draw (if not first discard)      |
| `Discard`        | Active seat must discard (or win on tsumo)       |
| `Reaction`       | Other seats may ron / call / pass (in priority)  |
| `CallResolve`    | Complete chii/pon/kan; new discard or rinshan    |
| `RiichiDeclare`  | Optional sub-phase after riichi declaration      |
| `HandEnd`        | Scoring or exhaustive draw                     |
| `MatchEnd`       | Game over (optional quit or target score)        |

All transitions emit **`Event`s** and validate **`Action`s** for the requesting `PlayerSlot` only.

### 3.3 Event log (online-ready)

```text
MatchId, RulesConfig, Seed
  → Vec<Event>   // append-only; apply() is pure
```

- **`Event`**: `Dealt`, `DoraRevealed`, `Drawn`, `Discarded`, `Called`, `RiichiDeclared`, `Won { … }`, `ExhaustiveDraw`, `ScoresAdjusted`, …
- **`Replay`**: in-memory match history (`rules_profile`, `rules_config`, `seed`, `events`); serializable via `serde` feature for tests. Post-v0 **file export/import** spec in `docs/REPLAY.md` (§11.4).
- **Future net layer** replays the same log on all peers or ships **actions** to an authoritative server that validates via `librrmj` — no protocol in v0, but **no hidden mutable globals**.

### 3.5 Extensible ruleset architecture

v0 ships **one** profile (`standard`), but the engine must not hardcode “the only ruleset” in `state/`, `action/`, or `event/`. Rule-specific behavior lives behind **`RulesProfile`**; the state machine calls the active profile for anything that can differ between rulesets.

**Design contract (from Phase 4 onward):**

```rust
// conceptual — names may differ in code
trait RulesProfile: Send + Sync {
    fn id(&self) -> RulesProfileId;
    fn legal_yaku(&self, config: &RulesConfig) -> &[YakuId];
    fn score_win(&self, ctx: &WinContext, config: &RulesConfig) -> ScoringResult;
    fn abortive_draws(&self, state: &HandState, config: &RulesConfig) -> Option<AbortiveDraw>;
    fn match_flow(&self) -> &dyn MatchFlowPolicy;
    // extend as needed; default impls for shared riichi behavior where sensible
}
```

**What stays profile-agnostic (shared engine):**

- Tile wall mechanics, deal order, turn rotation, meld shapes, reaction priority framework.
- `Action` / `Event` types (may gain profile-specific payloads later via enums, not ad-hoc fields).
- `Agent` loop, RNG injection, event log structure.

**What is profile-specific (per `rules/<name>/`):**

- Yaku detection and han assignment.
- Fu calculation and limit-hand table.
- Payment matrix and rounding.
- Enabled abortive draws and exhaustive-draw payments.
- Match length / round structure (e.g. east-only vs hanchan) when variants diverge.

**Registry and discovery:**

- `RulesProfileId` enum + `RulesRegistry::get(id) -> &dyn RulesProfile`.
- v0: registry contains only `Standard`; tests assert unknown IDs error cleanly.
- Optional **Cargo features** per heavy variant later (e.g. `rules-three-player`) so CI default stays lean.

**Anti-patterns (banned):**

- `if rules_profile == ThreePlayer` (or similar) outside `rules/` modules.
- Duplicate state machines per ruleset.
- Scoring or yaku tables referenced directly from `state/` or `rrmj-tui`.

**Adding a new ruleset later (checklist):**

1. Add `librrmj/src/rules/<name>/` with `mod.rs`, `yaku.rs`, `scoring.rs`, `tests.rs`.
2. Implement `RulesProfile`; register in `RulesRegistry`.
3. Add `RulesProfileId` variant + default `RulesConfig` if needed.
4. Document in `docs/RULES_<name>.md`; add scoring fixtures and at least one full-hand integration test.
5. Wire TUI/CLI profile picker (post-v0); until then, profile is selectable only via API/tests.

**v0 verification:** standard riichi passes all tests **through** `RulesProfile` dispatch, not bypassing it.

### 3.6 Agent abstraction

```rust
// conceptual — names may differ in code
trait Agent {
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action;
}
```

- **`PlayerView`**: what a seat is allowed to see (concealed tiles only for self; opponents show discards, melds, riichi flags, tile counts).
- **`CpuAgent`**: `Easy` / `Medium` / `Hard` behind `librrmj/ai/` + `ai` feature.
- **`HumanAgent`**: `rrmj-tui` blocks on input, returns `Action`.
- **`RemoteAgent` (later)**: receives `Action` from network task; same trait boundary.

`Match::step()` loop: determine legal actions for current actor → agent chooses → engine applies or rejects.

---

## 4. Rules engine architecture

### 4.1 Layering

1. **Primitives** (`tile`, `hand`, `wall`) — no phase knowledge.
2. **Legality** (`action`) — given snapshot + active **`RulesProfile`**, list legal `Action`s.
3. **Application** (`event`, `state`) — apply one action → one or more events → new snapshot; profile consulted for rule checks inside transitions.
4. **Scoring** (`rules/<profile>/`) — on win, delegate to `RulesProfile::score_win`; shared result types in `scoring/`.
5. **Match flow** (`game`) — rotate dealer, carry scores, detect match end; policy hooks from `RulesProfile::match_flow()`.

### 4.2 Winning and waits

- Standard win detection (4 melds + pair / seven pairs if enabled).
- **Wait calculator** for riichi declaration and AI — cached per hand where useful.
- **Furiten** tracking (discard furiten, riichi furiten, temporary).

### 4.3 Dora

- Indicator tiles, ura dora after riichi win, kan dora reveals.
- Aka dora as optional rules flag.

### 4.4 Testing strategy for rules

- **Table-driven** yaku/scoring fixtures in `librrmj` integration tests (input hand + context → expected han/fu/points).
- **Property tests** (optional `proptest` feature): random legal play sequences never panic; tile conservation invariant (136 tiles accounted for).
- **Replay regression**: check in `examples/replays/*.jsonl` for known hands.

---

## 5. AI architecture

### 5.1 Design constraints

- AI lives in **`librrmj`** under `ai/` (feature **`ai`**) so TUI and future Bevy share opponents.
- AI is **heuristic + search**, not ML for v0.
- Each difficulty is a separate submodule with **`tests.rs`** using fixed seeds.

### 5.2 Difficulty tiers (target behavior)

| Tier     | Behavior (summary)                                                                 |
| -------- | ---------------------------------------------------------------------------------- |
| **Easy** | Random among legal actions; occasional obvious tsumo/ron; no defense.              |
| **Medium** | Basic tile efficiency (shanten reduction); take obvious yaku; simple safe discard vs riichi opponents. |
| **Hard** | Better efficiency (ukeire / acceptance); suji/kabe-style defense; riichi timing; rudimentary push/fold. |

### 5.3 AI interface

- `AiConfig { difficulty, think_time_hint }` — hint only; engine remains synchronous (TUI can show “thinking…” while calling `decide`).
- Optional **deterministic** “personality” seed per CPU seat for variety without breaking replays.

---

## 6. TUI architecture (`rrmj-tui`)

### 6.1 Stack

- **`ratatui`** + **`crossterm`** (or `ratatui` default backend) — no alternate GUI stack in v0.
- **60 fps not required** for normal play; redraw on state change or input. Short **presentation animations** in ASCII mode may tick at a modest fixed rate (e.g. 10–15 Hz) for the duration of a scripted effect only.
- **`librrmj` has zero presentation concerns** — no colors, themes, or key names in the engine.

### 6.2 Launch flow (v0)

On startup the TUI opens the **main menu** first (not the table). Minimum entries:

| Entry        | Action |
| ------------ | ------ |
| Logo / title | Branding only (ASCII or styled text) |
| Start game   | Opens **new-game setup** (see below) |
| Settings     | Theme, default difficulty, config paths |
| Exit         | Quit cleanly |

**New-game setup** (after Start game):

- Per seat (East → North): **Human** or **CPU**.
- For each CPU seat: difficulty (**Easy / Medium / Hard**).
- Optional: player seat preference (which seat the human occupies).
- Confirm → build `Match` from `librrmj` and enter table screen.

Additional main-menu entries (post-v0 or stretch): **Play replay**, **Rules profile** picker.

### 6.3 Screens (in-match and post-hand)

| Screen         | Purpose |
| -------------- | ------- |
| Table          | Hands (concealed for player), rivers, melds, dora, scores, turn indicator |
| Call menu      | Chi / pon / kan / ron / pass when in `Reaction` |
| Discard select | Choose tile to discard / riichi / tsumo |
| Hand result    | Scoring breakdown, continue |
| Match summary  | Final placements |

### 6.4 Input model — hotkeys only

- **Every** TUI action is reachable via keyboard hotkeys (no mouse required).
- Keys map to UI intents first; table actions map to `Action` candidates; **`librrmj` validates** legality.
- Show **only legal** options in menus (no “illegal move” spam).
- **Modal overlays** (help, rules reference) capture input until dismissed; game state does not advance underneath.

### 6.5 Overlays (always available in-game)

| Overlay | Default key | Purpose |
| ------- | ----------- | ------- |
| Keybind help | `h` | Full-screen list of **all** current bindings (navigation, table, menus, overlays). Must work from any screen including modals that allow it. |
| Rules / yaku reference | `?` (or `y` if `?` is awkward on some layouts) | Scrollable reference: yaku list with han values, brief rule notes (dora, riichi, furiten, exhaustive draw). Content sourced from TUI strings mirroring `docs/RULES.md` — not duplicated in `librrmj`. |

Both overlays are **presentation**; engine types stay unchanged.

### 6.6 Theming (`rrmj-tui` only)

- **Themes** control ratatui colors/styles (borders, selected tile, riichi indicator, score emphasis).
- Shipped built-ins: at least **default** (dark) and one alternate (e.g. **high-contrast**).
- Theme name in `config.toml`; optional per-theme override files under `$XDG_CONFIG_HOME/rrmj/themes/`.
- **`ascii_mode`** in `config.toml` (default **`true`**): tile/table rendering uses plain ASCII glyphs (e.g. `2m`, `E`, `P`); themes apply colors and emphasis to those glyphs. A **`unicode`** (or `enhanced`) render mode remains available for terminals that support it — current block/box style — but ASCII is the default because it reads more clearly on more setups.
- **ASCII animations** (default **on** when `ascii_mode` is on; disable via `animations = false` in `config.toml`): colorful, theme-driven motion layered on top of ASCII glyphs — not a separate renderer. Effects are **client-side only**, triggered from applied `Event`s / visible state deltas; the engine does not wait on them. Target set for v0.1:
  - **Discard** — tile slides or “pops” from hand to river with a brief color trail.
  - **Call / kan** — meld assembly flash; kan reveals dora with a highlight sweep.
  - **Riichi** — stick deposit + pulsing riichi indicator on the seat.
  - **Draw / rinshan** — subtle flip or glow on the drawn tile in hand.
  - **Win** — short celebratory burst (score lines, winner seat, optional confetti-style `*`/`+` rain in theme accent colors).
  - **Turn / reaction** — active seat border or prompt color cycle so “who acts” is obvious.
  - Animations respect the active **theme palette** (primary, accent, danger/safe discard colors); high-contrast theme uses motion + bold more than hue shifts.
  - Input is **blocked only during short mandatory cues** (e.g. win banner); otherwise CPU steps and overlays behave as today. Unicode mode may ship with a **reduced** animation set or static emphasis only.
- `librrmj` does not define or parse theme data.

### 6.7 Config files (`rrmj-tui`)

| File | Path | Purpose |
| ---- | ---- | ------- |
| General settings | `$XDG_CONFIG_HOME/rrmj/config.toml` | Default CPU difficulty, theme name, `ascii_mode` (default on), `animations` (default on in ASCII mode), player seat preference |
| Keybinds | `$XDG_CONFIG_HOME/rrmj/keybinds.toml` | Full hotkey map; **sane defaults** baked into the binary when file is missing |
| CLI override | `--config`, `--keybinds` | Explicit paths |

`keybinds.toml` structure (conceptual):

- Sections: `global`, `menu`, `table`, `overlay`.
- Each binding: logical action name → key(s); support chord vs single key as needed.
- Unknown action names in TOML → parse error with line hint (tested in `rrmj-tui`).

### 6.8 TUI tests

- `rrmj-tui`: parse `config.toml` / `keybinds.toml`, default keymap completeness, action-name → `librrmj::Action` mapping — **no** full terminal driver required in CI.

---

## 7. Online-ready design (post-release; architect now)

Not implemented in v0, but **must** hold from Phase 1 onward:

| Principle              | Implementation in `librrmj`                                      |
| ---------------------- | ---------------------------------------------------------------- |
| Single source of truth | All state in `MatchState`; clients hold copies updated by events |
| Deterministic core     | Seeded wall + event apply                                        |
| Action validation      | Server (later) calls same `legal_actions()` / `apply()` as local |
| No UI in protocol      | Wire format is `Action` + `Event`, not screen diffs              |
| Transport-agnostic     | Future `rrmj-net` crate: WebSocket/QUIC; **not** in `librrmj`  |

**Suggested future model (document in `docs/ONLINE.md` after v0):**

- **Authoritative server** hosts `Match`; clients send `Action` proposals; server broadcasts `Event`s.
- **Spectator / replay**: distribute read-only event log.
- **Clock / ranking**: outside `librrmj`.

---

## 8. Quality gates

Whenever a phase is marked complete:

- `cargo fmt --check`
- `typos`
- `cargo deny check licenses` (`deny.toml` allow list populated for used crates)
- `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --no-default-features`
- `cargo test --workspace --all-features`
- `cargo doc --workspace --no-deps`

### 8.1 Test discipline

- Unit tests in **exactly one `tests.rs`** next to `mod.rs` per **directory module** (`state/tests.rs`, `tile/tests.rs`, `rules/standard/tests.rs`, …).
- **Forbidden**: extra sibling test modules such as `calls_tests.rs`, `win_tests.rs`, or `#[cfg(test)] mod tests` inside logic files — use section comments inside the single `tests.rs` instead (`// --- calls ---`, etc.).
- Integration tests under `librrmj/tests/` for full hands, scoring tables, AI smoke tests.
- `rrmj-tui`: lightweight tests for config parse and action mapping (no full terminal driver required).

### 8.2 CI

- Matrix: default / all-features / no-default-features on `build`, `fmt-clippy`, `test`, `doc`.
- Coverage job targets `librrmj` (primary) and `rrmj-tui` (secondary).

---

## 9. Phased steps

### Phase 0 — Workspace purge + skeleton

- [x] Remove legacy crates and obsolete scripts; align workspace `members` with `librrmj`, `rrmj-tui`.
- [x] Scaffold empty crates: `librrmj` exports `VERSION`, `RulesConfig::standard()`; `rrmj-tui` prints version and exits 0.
- [x] Wire **tracing** + `tracing-subscriber` in `rrmj-tui` only.
- [x] Update CI workflows for the new crate layout; fix `deny.toml` allow list for initial deps.
- [x] Rename coverage flags / package names in CI.

**Verify**: all gates in §8; `cargo build --workspace` succeeds.

### Phase 1 — Tiles, hand, wall primitives

- [x] `tile/`: `Tile`, suits, honors, red fives, compare/sort, display helpers (no UI strings in lib — use `Display` or compact `to_string` for logs).
- [x] `hand/`: concealed tiles, melds (chi/pon/kan), tile count invariants.
- [x] `wall/`: build 136-tile wall, shuffle with injected RNG, deal 13+1 to dealer, dead wall layout.

**Verify**: unit tests for tile ordering, deal counts, wall exhaustion indices.

### Phase 2 — Discard flow + turn order

- [x] `state/`: `HandState` with seats, current actor, discards per seat.
- [x] `action/`: `Discard`, `Draw` (internal), `Pass`.
- [x] `event/`: `Dealt`, `Drawn`, `Discarded`; apply updates state.
- [x] Turn rotation among four seats; simple “play until wall empty” loop without calls or wins.

**Verify**: integration test — scripted discards through N turns; tile conservation.

### Phase 3 — Calls (chi, pon, dakaikan)

- [x] `Reaction` phase and call priority (ron > pon/kan > chii).
- [x] Open melds update `hand/`; kan updates dora indicator (if rules say so).
- [x] After call, caller discards (except closed kan edge cases per rules).

**Verify**: table tests for each call type; rejection of illegal calls.

### Phase 4 — Wins, yaku, scoring + rules profile boundary

- [x] Introduce `RulesProfile` trait, `RulesProfileId`, `RulesRegistry`; **`standard`** as sole registration.
- [x] Win detection (tsumo/ron); `rules/standard/` yaku table per `docs/RULES.md`.
- [x] Fu calculation, han aggregation, limit hands, payment matrix (ron/tsumo, dealer/non-dealer) via profile.
- [x] Riichi declaration, riichi stick, dora/ura/aka in scoring.
- [x] `HandEnd` → score transfer; `ExhaustiveDraw` (tenpai/noten payments).
- [x] No rule-specific branches in `state/` outside profile dispatch.

**Verify**: extensive scoring fixtures; cross-check sample hands against manual calculations or known calculators; grep/architecture review — yaku/scoring logic only under `rules/standard/`.

### Phase 5 — Match flow + special draws

- [x] `game/`: east/south rounds, honba, renchan, dealer rotation.
- [x] Abortive draws (if enabled in `RulesConfig`).
- [x] Match end condition (e.g. south 4 ends, or optional “first to 30k” in config).

**Verify**: multi-hand integration test to completion with scripted wins/draws.

### Phase 6 — Agent loop + event log API

- [x] `agent/`: `Agent` trait, `PlayerSlot`, `PlayerView` (information hiding).
- [x] `Match::step()` / `apply_action()` public API for clients.
- [x] `Replay` struct: `rules_profile`, `rules_config`, `seed`, `events`; optional `serde` feature; round-trip test.
- [x] Stable in-memory replay API (`Replay::from_match`, `Replay::apply_all`) ready for post-v0 file format (§11.4).

**Verify**: drive full hand via `Vec<Action>` in test; replay from log matches live play.

### Phase 7 — AI: Easy + Medium

- [x] `ai/easy/`: random legal action with obvious win capture.
- [x] `ai/medium/`: shanten-based discard; basic call acceptance.
- [x] `AiConfig` wired into `Match` setup.

**Verify**: AI vs AI smoke test completes a hand without panic; easy always legal.

### Phase 8 — AI: Hard

- [x] `ai/hard/`: improved efficiency metric; defensive discard vs open riichi.
- [x] Tunable constants documented in module; no “magic” without comment.

**Verify**: hard beats medium >50% in short benchmark (seeded sim, optional statistical tolerance).

### Phase 9 — TUI vertical slice

- [x] App shell: **main menu** on launch (logo, start game, settings, exit).
- [x] **New-game setup**: per-seat human/CPU, CPU difficulty, confirm → `Match`.
- [x] Table view: hand, river, melds, dora, scores, turn indicator.
- [x] Call / discard / riichi / win menus from legal actions only; **all via hotkeys**.
- [x] Hand result screen + continue.
- [x] Default `keybinds.toml` + loader (`~/.config/rrmj/keybinds.toml`); `h` → full keybind help overlay.

**Verify**: manual playtest — menu → setup → complete one full hand; `h` shows complete binding list.

### Phase 10 — TUI polish + first release

- [x] `config.toml` (theme, defaults, `ascii_mode`, `animations`) + CLI `--config` / `--keybinds`.
- [x] **ASCII render mode** (default **`ascii_mode = true`**): tiles, melds, rivers, and table chrome drawn with plain ASCII; themes colorize glyphs (selected tile, riichi, dora, scores). Keep current Unicode/block-style layout as an alternate **`unicode`** (or `enhanced`) mode for capable terminals.
- [x] **ASCII animations** (default **on**): theme-colored motion for discard, calls/kan/dora, riichi, draw, win, and turn/reaction cues — driven by `Event`s in `rrmj-tui` only; `animations = false` disables. Implement as a small `ui/anim/` timeline (queued effects, fixed tick during playback).
- [x] **Theming**: built-in palettes; theme selectable in settings; works in both ASCII and Unicode render modes; animation accents read from theme tokens.
- [x] **Rules / yaku reference** overlay (dedicated hotkey); content aligned with `docs/RULES.md`.
- [x] README: build, run, config paths, `ascii_mode` vs Unicode rendering, animations toggle, rules pointer, difficulty description.
- [x] CHANGELOG; tag **v0.1.0**.

**Verify**: full §8 gates; fresh clone `cargo install` path documented; missing keybinds file uses sane defaults; default config uses ASCII mode with at least discard + win animations visible; `animations = false` restores static ASCII; themes apply visibly on a plain terminal.

---

## 10. Definition of done (v0.1 / first release)

- [x] `librrmj` plays full 4-player riichi per `docs/RULES.md` locally with event log API.
- [x] `rrmj-tui` supports a full match vs 3 CPU opponents at **easy / medium / hard**.
- [x] No ratatui/crossterm in `librrmj`; no game rules in `rrmj-tui`.
- [x] CI green on default / all-features / no-default-features; docs build.
- [x] **Online multiplayer not required**; event log + `Agent` trait demonstrate extensibility.
- [x] **One rules profile** (`standard`) implemented, but all scoring/match-policy calls go through **`RulesProfile`** (§3.5).
- [x] **Bevy client not required**; crate boundary documented for future `rrmj-bevy`.
- [x] **Replay file export/import not required**; in-memory `Replay` + serde groundwork in place.

---

## 11. Post-first-release roadmap (separate plans)

### 11.1 Online multiplayer (`rrmj-net` + `docs/ONLINE.md`)

- [ ] Specify wire messages (`Action`, `Event`, snapshot sync).
- [ ] Authoritative server binary or library feature.
- [ ] `RemoteAgent` + latency-aware UI in `rrmj-tui`.

### 11.2 Bevy GUI (`rrmj-bevy`)

- [ ] New workspace member; same `Match` / `Agent` wiring as TUI.
- [ ] Asset pipeline for tiles/table; animation for discards/calls.
- [ ] Dedicated `docs/BEVY_PLAN.md` when work starts (not before v0.1 ships).

### 11.3 Additional rulesets

- [ ] New profiles via §3.5 checklist (e.g. three-player, casual yaku toggles, house-rule packs).
- [ ] `RulesProfileId` picker in TUI/CLI; per-profile `RulesConfig` presets in `examples/`.
- [ ] Optional Cargo features for heavyweight variants; CI keeps `standard` on default matrix.

### 11.4 Match replay export / import

Ship a **stable, versioned replay format** so matches can be saved and played back in rrmj (TUI first; Bevy later).

**Format (spec in `docs/REPLAY.md`):**

- File extension: `.rrmj` (or `.rrmj.json` if plain JSON).
- Top-level **`format_version`** for forward-compatible migrations.
- **`rules_profile`** + **`rules_config`** — replay must load under the same profile (clear error if missing/unknown).
- **`seed`**, **`players`** (display names, agent kind per seat), **`events`** (canonical event log; preferred over action log for spectators).
- Optional metadata: timestamp, client version, match title.

**Library (`librrmj`, `serde` feature):**

- [ ] `Replay::to_file` / `Replay::from_reader` with schema validation.
- [ ] `ReplayPlayer` — step forward/back, seek to event index, derive `MatchState` at any point.
- [ ] Round-trip tests: live match → file → `ReplayPlayer` state equals original at each step.

**Clients:**

- [ ] `rrmj-tui`: export after match (or from pause menu); **Play replay** from main menu; playback controls (step, play/pause, speed).
- [ ] CLI hook (optional): `rrmj-tui replay path/to/file.rrmj`.

**Out of scope for replay v1:** editing replays, compressed binary encoding, importing third-party formats (Mahjong Soul, etc.) — may follow as separate adapters.

### 11.5 Optional enhancements

- [ ] Stronger AI (expectimax / neural — only if needed).

---

## 12. Dependency policy

- **Edition**: `2024` (workspace).
- **Versions**: `x.y` or `x` in manifests; lockfile committed.
- **Health**: avoid archived / unmaintained crates.
- **`librrmj`**: keep lean — `thiserror`, `tracing`; `rand` + `rand_chacha` for RNG; optional `serde` / `proptest` behind features.
- **`rrmj-tui`**: `ratatui`, `crossterm`, `clap`, `toml`, `tracing-subscriber`.
- **Banned from `librrmj`**: `ratatui`, `crossterm`, `bevy`, `tokio` (sync engine unless a later phase proves need), network stacks.
- **Heavy deps**: justify in PR; behind features.

---

## 13. Document maintenance

Update this plan when:

- ruleset or yaku list changes — update `docs/RULES.md` (or `docs/RULES_<profile>.md`) first
- new `RulesProfile` added — follow §3.5 checklist
- replay format changes — bump `format_version` in `docs/REPLAY.md`
- crate split or feature set changes
- online or Bevy work starts — add/update dedicated sub-plans
- §1.2 code-comment rule changes

---

## Revision history

| Date       | Change                                      |
| ---------- | ------------------------------------------- |
| 2026-06-08 | Initial rrmj plan                          |
| 2026-06-08 | §3.5 extensible rulesets; §11.4 replay format |
