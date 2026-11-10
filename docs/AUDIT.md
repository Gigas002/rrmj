# Audit

The pre-release audit: a hardcore test-and-fix pass over `librrmj` and `rrmj`
before the first tagged release. It has ten parts, run in the order below,
then a wrap-up (§11):

| §  | Part                           | Main target                         |
| -- | ------------------------------ | ----------------------------------- |
| 1  | Baseline                       | the whole workspace                 |
| 2  | Ruleset correctness            | `librrmj/src/rules/`, `state/`      |
| 3  | Gameplay and match flow        | `state/`, `game/`, `rrmj/src/app/`  |
| 4  | AI behavior                    | `librrmj/src/ai/`                   |
| 5  | Bugs and robustness            | everything; panics, edge states     |
| 6  | Performance                    | AI, recommendations, render loop    |
| 7  | UI correctness and usability   | `rrmj/src/ui/`, `input/`, `app/`    |
| 8  | Security                       | file input, config, deps            |
| 9  | Features against the plan      | `docs/PLAN.md` against the code     |
| 10 | Docs                           | `README.md`, `docs/`, `examples/`   |

Each part is a list of steps. A step says what to check, how, and when it
passes. The findings go into the table at the end of each part. Each finding
says what was found, how serious it is, and what was done:

- **fixed**: fixed within the audit step (small, local, tested).
- **accepted**: left as is, with the reason.
- **follow-up**: too big for the audit; becomes a new step in
  `docs/PLAN.md` §10, done before the release.

Severity:

- **high**: a wrong rules result a player would notice (wrong winner, wrong
  score, illegal action offered or legal one missing), a crash or hang in
  normal play, lost or corrupted saves.
- **medium**: a crash, hang or unbounded allocation from a hand-edited or
  hostile file; a wrong result in a rare rules case; a UI state the player
  can't leave; AI that breaks the game's pace or obviously misplays.
- **low**: polish, wording, hardening, a doc that's out of date.

**Leads** under a part are things already seen while writing this plan. They
aren't findings until checked: confirm each one against the code, then record
it in the table (or note it was a false lead).

Working rules:

- Work on the `chore/audit` branch. One commit per part, named
  `chore(audit): §N <part>`; a fix too big for its part gets its own commit.
- Every **fixed** finding gets a regression test: a unit test in the
  module's `tests.rs`, or a scenario JSON under `examples/scenarios/` with
  `assertions` (see `docs/DEBUG_SCENARIOS.md`).
- No rules logic moves into `rrmj` to fix a UI bug (PLAN §1.2, §3.4).
- Rerun the §8 gates of `docs/PLAN.md` at the end of each part.

---

## 1. Baseline

Get a known-good starting point so later findings aren't confused with an
already broken build.

- [x] **1.1** Run every PLAN §8 gate on a clean checkout and record the
  result. Also run plain `cargo test --workspace` (default features), since
  CI runs that too.
- [x] **1.2** Build `rrmj` three ways and confirm each starts and reaches
  the main menu: default features, `--no-default-features` (no `ai`: what
  happens when a seat is set to CPU?), and `--features debug-menu`.
- [x] **1.3** Record sizes and counts for later comparison: release binary
  size, number of tests, number of scenarios, `cargo build --release` time.
- [x] **1.4** Grep for panics outside tests (`unwrap`, `expect`, `panic!`,
  `unreachable!`, slice indexing on input-derived indices). Classify each:
  invariant (fine, document why), or reachable from outside data or
  player input (finding for §5/§8). Hotspots seen when writing this:
  `rrmj/src/app/mod.rs` (11), `librrmj/src/ai/strategy.rs` (7),
  `librrmj/src/state/hand_state.rs` (6).
- [x] **1.5** Confirm there is no `unsafe` (none found when writing this).

Done 2026-11-09 at `64e6323`, rustc 1.99.1, Linux, 32 cores.

**1.1 Gates.** All PLAN §8 gates passed at the start: fmt, typos,
`deny check licenses`, clippy (workspace, no-default and all features),
tests (default, no-default, all features), `cargo doc` with
`RUSTDOCFLAGS="-D warnings"`. But the workspace gates never built the
minimal `librrmj`: see the first finding.

**1.2 Builds.** Each release build was started in a pseudo-terminal with an
empty `XDG_CONFIG_HOME`/`XDG_DATA_HOME` and quit with `q`. All three reach
the main menu, create no files and restore the terminal (alternate screen
left, cursor shown, exit 0). `debug-menu` adds the Debug item. The
`--no-default-features` build starts a match with three CPU seats and the
CPUs play, because the `ai` feature of `rrmj` doesn't remove anything
(finding below).

**1.3 Baseline numbers.**

| What | Value |
| ---- | ----- |
| Clean `cargo build --release -p rrmj` | 4.8 s |
| `rrmj` release binary | 4.5 MB (3.5 MB stripped); no `[profile.release]` settings |
| `rrmj --help` | 0.65 ms (median of 5) |
| Tests, `--all-features` | 189: `librrmj` unit 139, `tests/scenarios.rs` 5, `rrmj` 45, doctests 0 |
| Tests, `librrmj` alone, no features | 109 |
| Scenario files | 50 (`examples/scenarios/`) |
| `librrmj` unit tests, debug build | 73 s, of which `ai::tests::hard_beats_medium_in_short_benchmark` 73 s and `ai_vs_ai_smoke_completes_without_panic` 14 s (run in parallel); the rest are under 0.5 s. The whole `ai::tests` set takes 5.7 s in release. |

**1.4 Panic sites.** 51 outside tests. Invariants (fine): the UI `expect`s
on state the current screen guarantees (`app/`), built-in key specs
(`input/keybinds`), the fixed wall layout, `unreachable!` on
`seat % 4`-style matches, the AI's `min`/`max`/`choose` on lists the
discard phase guarantees non-empty, `RulesRegistry::get` for the one
profile. Reachable from a file: the pending-reaction `expect`s in
`state/hand_state.rs`, which a snapshot sets directly (findings below).
`ai/setup.rs:115` is safe: `ai_config_for` always returns `Some` for a
CPU seat.

**1.5** No `unsafe` in either crate.

| Finding | Severity | Status |
| ------- | -------- | ------ |
| The minimal `librrmj` didn't compile under clippy `-D warnings`: `MeldKind::parse_str` (used only by the serde impl) and `ReactionState::responses`/`from_responses` (no callers at all) were dead code without `serde`. No gate caught it, because `--workspace --no-default-features` still builds `librrmj` with `ai` and `serde` (Cargo unifies them with `rrmj`'s dependency features). `parse_str` is now gated on `serde`, the two unused methods are removed, and CI (clippy matrix, tests), PLAN §8 and ARCHITECTURE §8 now check `-p librrmj --no-default-features` too. All feature combinations of `librrmj` are clippy-clean. | medium | Fixed |
| `rrmj`'s `ai` feature does nothing: `rrmj` depends on `librrmj` with `ai` and `serde` always on (and `librrmj`'s `serde` implies `ai`), so `--no-default-features` builds the same game. `librrmj`'s `test-util` feature gates no code. Decide per §9.5: drop them, or make them real. | low | Follow-up (§9.5) |
| A crafted recording panics the engine: a pending `Kakan` reaction with `kakan_meld_index: null` panics at `state/hand_state.rs:753` once the last seat answers, and an index past the seat's melds panics at `hand_state.rs:606`. `restore` also accepts `discarder: 7`, `current_actor: 9` and `dealer: 5` without error. Shown with a throwaway probe test on `chankan_ron.json`. A shared scenario can crash `rrmj`. | medium | Follow-up (§8.2: validate every snapshot index on restore) |
| `rrmj`'s logger (`tracing_subscriber::fmt()`, default writer) prints to stdout, the terminal the TUI draws on. At the default `info` level only save failures log, so a failed match-end replay write draws a line over the table and is otherwise never shown; `RUST_LOG=debug` garbles the screen. Log to a file under `$XDG_STATE_HOME/rrmj/` (or nowhere by default) and show save errors in the UI. | medium | Follow-up (§5.7) |
| The match-end replay is written on a detached thread (`save::write_recording_async`). Quitting right after a match ends can exit before the write is done, losing the replay without a message. | medium | Follow-up (§5) |
| `q` (and `m`) leave a match in progress at once, with no prompt and no save. Since Phase 14.3 removed autosave, one key loses the whole match. | high | Follow-up (§3.10) |
| The two AI benchmark tests make every debug `cargo test` take 73 s. Hard's decisions are slow in debug builds; see §6.1. Options: `#[ignore]` them and run them in a release CI job, or set `opt-level` for `librrmj` in `[profile.test]`. | low | Follow-up (§6.1) |

---

## 2. Ruleset correctness

The engine is the product: a scoring or legality error is a release blocker.
The reference is `docs/RULES.md`. Where `RULES.md` is silent or ambiguous,
decide the rule, write it into `RULES.md`, then make the engine match.

### 2.1 Tiles, wall, deal

- [x] **2.1.1** 136 tiles (with the configured aka fives replacing normal
  fives, not added), dead wall 14, dora indicator position, kan dora order,
  rinshan draw order, haitei tile. Check against `RULES.md` and with a
  seeded wall dump.
- [x] **2.1.2** Tile conservation after every action of a long random match
  (all seats CPU, many seeds): wall + dead wall + hands + melds + rivers =
  136 at every step. `validate_tile_conservation` exists; call it after each
  `apply_action` in a test harness.
- [x] **2.1.3** Determinism: same seed + same actions = same event log and
  same final scores, run twice and compared.

### 2.2 Calls and legality

For each, check `legal_actions()` offers exactly the legal set, no more, no
less, for the seat that must act.

- [x] **2.2.1** Chi: only from kamicha; all three shapes; with aka fives
  (aka and normal five offered as distinct choices where it matters);
  no chi on the last discard (houtei) and no chi when the wall is empty.
- [x] **2.2.2** Kuikae (swap-calling): after chi/pon, discarding the called
  tile or the other end of the same sequence. `RULES.md` doesn't mention it;
  decide and document.
- [x] **2.2.3** Pon and open kan from any seat; priority ron > pon/kan > chi;
  a pon and a chi on the same discard; two seats able to pon (impossible,
  but check the code doesn't assume it).
- [x] **2.2.4** Closed kan and kakan: not on the last tile, not after the
  fourth kan except the one completing suukantsu; closed kan during riichi
  only when it doesn't change the wait (and the decomposition); rinshan
  draw; new dora timing (closed kan at once, open kan/kakan after discard if
  that's the chosen rule; document it).
- [x] **2.2.5** Chankan on kakan; kokushi ron on closed kan (allowed or
  not; document); chankan with double ron.
- [x] **2.2.6** Riichi: needs closed hand, tenpai, 1000 points, at least 4
  tiles left in the wall (check the exact threshold); riichi discard must
  keep tenpai; riichi stick is paid only once the discard isn't ronned;
  after riichi, only tsumogiri, tsumo, ron and allowed closed kan.
- [x] **2.2.7** Furiten: permanent (own discard in the wait), temporary
  (passed on a winning tile until own next discard), riichi (passed after
  riichi: furiten to the end of the hand); tsumo still allowed in all three.
  Waits include tiles that complete the hand with no yaku: furiten still
  applies.
- [x] **2.2.8** Win with no yaku: neither ron nor tsumo offered. Dora alone
  isn't a yaku.
- [x] **2.2.9** Double/triple ron by `RulesConfig`; when `triple_ron` is
  off, three rons = abortive draw (sanchahou) or head-bump? Decide,
  document, test. Who gets the riichi sticks and honba on double ron
  (closest seat by turn order).

### 2.3 Yaku, fu, limits, payments

- [x] **2.3.1** One test per yaku in `RULES.md`'s table, open and closed
  where both exist, checked for han. The yaku catalog lives in
  `librrmj/src/rules/standard/tests.rs` now (`WinCase`); PLAN still
  calls it `cheatsheet.rs` (see §10).
- [x] **2.3.2** Decomposition choice: a hand with several readings scores
  the highest (e.g. ryanpeikou vs chiitoitsu, sanankou vs sequences,
  pinfu vs a kanchan reading of the same tiles, ittsu vs other splits).
- [x] **2.3.3** Fu, table-driven against hand-computed values, including:
  open pinfu-shape ron = 30; pinfu tsumo = 20 (no +2); closed ron +10;
  shanpon ron makes that triplet open for fu (and for sanankou);
  double-wind pair (+2 or +4; document); kan fu open/closed,
  simple/terminal; tanki/kanchan/penchan +2; round up to 10; chiitoitsu
  25 fixed.
- [x] **2.3.4** Limits: mangan at 5 han, and at 4 han 40+ fu / 3 han 70+
  fu; kiriage mangan when `kiriage` is on; haneman/baiman/sanbaiman;
  counted yakuman (13+ han) if supported, otherwise document it isn't;
  double yakuman and yakuman stacking (e.g. daisangen + tsuuiisou)
  per `RULES.md`.
- [x] **2.3.5** Yaku that `RULES.md` should list but may not: tsuuiisou,
  chinroutou, honroutou, sanankou, sankantsu, sanshoku doukou,
  shousangen, chankan, nagashi mangan. For each: implemented and tested,
  or listed in `RULES.md` as deliberately not supported.
- [x] **2.3.6** Payments: dealer/non-dealer, ron/tsumo, honba (300 per
  honba on ron, 100 from each on tsumo), riichi sticks to the winner,
  rounding up to 100 per payment. Compare against a published table for
  every han/fu cell used (1–4 han × 20–110 fu).
- [x] **2.3.7** Dora: indicator wraps (9→1, North→East, Chun→Haku), aka
  counted once each, ura only for riichi winners, kan ura revealed.

### 2.4 Draws and match flow

- [x] **2.4.1** Exhaustive draw: tenpai/noten payments (1000/1500/3000
  split); a hand that's tenpai only on a tile the player holds all four of
  (karaten) — tenpai or not (document); honba +1; dealer keeps the seat
  when tenpai.
- [x] **2.4.2** Abortive draws, each behind its `RulesConfig` flag: nine
  terminals (only on the first uninterrupted turn), four winds, four kongs
  (by two or more players), four riichis (after the fourth riichi stick is
  placed). Honba +1, dealer stays.
- [x] **2.4.3** Renchan and rotation: dealer win or dealer tenpai keeps
  the seat; otherwise rotate; honba resets on a non-dealer win.
- [x] **2.4.4** Match end: hanchan/tonpuusen length; `target_score`;
  south (west) extension if nobody reaches the target, if supported;
  bust (below zero) ends the match or not (document); all-last dealer
  stops when in first place (agari-yame), or not; leftover riichi sticks
  at match end go to first place or disappear (document).
- [x] **2.4.5** Ties in final placement broken by seat order.

### 2.5 Fuzzing the engine

- [x] **2.5.1** A test (or `#[ignore]`d long test run by hand) that plays
  thousands of seeded matches with four CPU seats of mixed difficulty and
  asserts on every step: tile conservation, scores sum to
  `4 × starting_points` minus sticks on the table, no panic, every
  chosen action was in `legal_actions()`, every match ends.
- [x] **2.5.2** A random-action variant: seats pick uniformly from
  `legal_actions()` (exercises calls and kans the real AI rarely makes).
- [x] **2.5.3** For every `Won` event in those runs, recompute the score
  from the final hand with `score_win` and compare. Log any hand where the
  win had no yaku.

### 2.6 Results

Tools added: `librrmj/tests/simulation.rs` plays seeded matches (six seat
mixes of Easy/Medium/Hard CPUs and uniform-random seats, four rule
variants) and checks after every action: a legal action exists, the chosen
action is legal and applies, the full 136-tile multiset is conserved
(`validate_tile_conservation` now compares tiles, not just a count),
points plus table sticks are conserved; on every win: a yaku, han adds up
(yaku with open −1 + dora), fu rounded, winner gains; draws sum to zero;
final scores sum to the starting total. A short run is in the normal test
suite, determinism is a test, the long run is `#[ignore]`d (see the file
header). Yaku and fu are now computed by `rules/standard/eval/`, which
scores every reading of the hand and keeps the best; fixtures and the fu
table were corrected where they encoded the old behaviour, and a payment
test checks the published table. RULES.md and the in-app rules pages were
rewritten to match. Long run (`RRMJ_SIM_GAMES=2000`, release, one thread,
38 min, 2026-11-10): passed. 2000 matches, 16 872 hands, 5.6 M actions;
7793 rons, 2473 tsumos, 8660 exhaustive draws, 12 abortive (four winds 9,
four kongs 2, nine terminals 1); 73 875 chi, 45 425 pon, 2623 open kans,
2457 kakan, 545 closed kans, 4042 riichi. Every non-yakuman yaku was won at
least once except ryanpeikou and renhou; among yakuman only kokushi (28)
appeared. Those are covered by the unit tests and scenarios instead.
Highest win 14 han.

Rule decisions taken (standard Tenhou / Mahjong Soul behaviour unless noted;
all written into RULES.md): kuikae banned; no kan right after a call; no
calls on houtei or while in riichi; riichi needs ≥ 4 live tiles, tsumogiri
only, closed kan only if waits don't change; honba +1 on every draw;
abortive draws keep the dealer; all-last renchan plays on unless the dealer
leads; bust ends the match; leftover sticks to first place; nine terminals
for any seat's first uninterrupted turn; four kongs only across players;
double-wind pair 4 fu; karaten counts as tenpai; multiple-ron bonus to the
closest winner; triple ron off = head bump (no sanchahou, kept from before);
renhou 5 han (kept); no nagashi mangan, counted or double yakuman, pao,
west extension; dora from kans revealed at once.

| Finding | Severity | Status |
| ------- | -------- | ------ |
| Kans broke the wall: rinshan draws came from the dora-indicator slots and the live wall never shrank, so tile counts drifted (137 after an open kan) and haitei came late. Rinshan now draws from `dead[10..14]`, each kan shortens the live wall, at most four kans. | high | Fixed |
| Wins without a yaku were accepted (`can_win` checked only the shape), e.g. yakuless open ron. | high | Fixed |
| `apply` didn't check legality: kans with an empty live wall, calls on the last discard, any discard during riichi, calls during riichi. `apply` now requires the action to be in `legal_actions_for`, with specific errors. | high | Fixed |
| Riichi was never offered: tenpai was tested on the 14-tile hand. | high | Fixed |
| The AI's shanten calculator was wrong, so CPUs didn't build hands; defence was only a tie-breaker. Real shanten/ukeire; a defending seat now puts safety first. | high | Fixed |
| Points not conserved: on ron the discarder also paid the riichi sticks, and every winner of a double ron got honba and sticks. | high | Fixed |
| Red fives didn't match plain fives in win shapes, so hands holding aka couldn't win (and recommendations offered a tsumo on a red five not in hand). | high | Fixed |
| A dragon or value-wind *pair* counted as yakuhai; two value triplets or a double wind counted once. | high | Fixed |
| Fu used the minimum-fu reading, rounded up only with `kiriage`, gave +2 and closed fu to shanpon ron triplets; yaku could combine across incompatible readings. Replaced by per-reading evaluation (best payout). | high | Fixed |
| Closed kans made the hand open: no riichi, no menzen tsumo, open −1 han. | high | Fixed |
| Round wind was hard-coded to East for yakuhai, pinfu and fu. | high | Fixed |
| 43 of 50 scenario files held impossible tile sets (duplicated or missing tiles); only the count was checked. Files repaired; `validate_tile_conservation` checks the multiset. | high | Fixed |
| Missing yaku: sanankou, sankantsu, sanshoku doukou, shousangen, honroutou, tsuuiisou, chinroutou. | medium | Fixed |
| No kuikae; a kan straight after a pon was allowed. | medium | Fixed |
| Chiitoitsu accepted four of a kind as two pairs. | medium | Fixed |
| Exhaustive draw with four tenpai divided by zero. | medium | Fixed |
| Chiihou didn't require an uninterrupted first go-around; yakuman listed riichi/ippatsu/etc. beside them. | medium | Fixed |
| Match flow: all-last always ended (even on dealer repeat), no bust, leftover sticks vanished, honba reset on a draw rotation, abortive draws could rotate the dealer. | medium | Fixed |
| Nine terminals was dealer-only (and refused by `apply` for others); four kongs aborted with all four kans in one hand; four winds ignored calls. | medium | Fixed |
| `kiriage` meant fu rounding (always on in riichi); it now means kiriage mangan. In-app point table had wrong 70 fu and 3 han 60 fu cells and wrong examples. `yaku_lines` showed closed han for open hands. | medium | Fixed |
| A riichi declaration discard that is ronned still costs the stick and keeps the riichi; standard play voids it. Needs the stick deferred in both live play and replay. | medium | Follow-up |
| Scenario assertions only check that listed actions/yaku are present; they can't assert an action is absent (e.g. no Ron when furiten), and the scenario test scores with `RulesConfig::standard()` instead of the recording's config. | medium | Follow-up (§8) |
| Easy (and random) seats call everything and almost never win (yakuless open hands). | medium | Follow-up (§4) |
| `calls::kamicha(discarder)` returns the seat *after* the discarder (the caller whose kamicha is the discarder); the name reads backwards. | low | Follow-up (§9) |
| Abortive draws are rare in simulation (none in 480 matches before the fixes); covered by scenarios and unit tests instead. | low | Accepted |

---

## 3. Gameplay and match flow (TUI ↔ engine)

Play real matches in the TUI and check that what the player sees and can do
follows the engine.

- [ ] **3.1** Play at least one full hanchan at each difficulty as the
  human, plus one with the human in each seat (0–3). Note anything odd.
- [ ] **3.2** Action menus show only `legal_actions()`, all of them:
  compare the actions bar with the engine's list in a debug log for every
  decision of one match.
- [ ] **3.3** Reaction window: when several calls are possible on one
  discard (ron + pon + chi), each is offered and picking one resolves by
  priority against CPU calls on the same tile.
- [ ] **3.4** Riichi flow: declaring, choosing the discard, the stick
  shown, auto-tsumogiri afterwards, tsumo/ron prompts while in riichi.
- [ ] **3.5** Timers (`rrmj/src/app/timers.rs`): turn timer expiry,
  response timer expiry, timers off (0), timers while an overlay is open,
  while paused, and across a reaction that the CPU resolves first.
  - Lead: on discard timeout the code discards `menu.discards.first()`;
    check that's the drawn tile (tsumogiri), not the leftmost tile of the
    hand.
  - Lead: on discard timeout it declares tsumo if possible; on reaction
    timeout it always passes, even with ron available.
    `examples/config.toml` says the response timer does "auto-pass (or
    tsumo if win)". Decide the behavior (auto-win on timeout, or never),
    make the code and the comment agree.
- [ ] **3.6** Hand end and match end: hand result popup matches the
  `ScoringResult` (yaku, han, fu, limit, payments, deltas); the scores
  overlay matches the engine's scores; draws show who was tenpai.
- [ ] **3.7** Saves: save mid-hand, quit, load, continue to the end; the
  file becomes `finished` and shows under Replays and not under Load game.
  Save during a reaction window and during a CPU turn. Load with a
  different human seat.
- [ ] **3.8** Replays: step forward and back through a full match, seek to
  each hand boundary, switch viewed seat; the board at each step equals the
  board during the live match.
- [ ] **3.9** Scenarios: every file in `examples/scenarios/` loads in the
  debug menu and plays on; player scenarios from `scenarios_dir` load;
  import (`i`) copies the file and it then lists.
- [ ] **3.10** Leaving: `m` (main menu) and `q` (quit) mid-match. The
  keybind file says `m` "does not save"; check whether either asks for
  confirmation before discarding a match in progress.

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 4. AI behavior

`librrmj/src/ai/`: Easy (random + obvious wins), Medium (shanten), Hard
(efficiency + defense). The bar for a release: legal, never stuck, sensible
at each tier, and each tier clearly stronger than the one below.

- [ ] **4.1** Legality and termination: covered by §2.5; also check every
  `Agent::decide` returns an action from `legal_actions` even for odd
  states (riichi with a closed kan option, last tile, four kans on table).
- [ ] **4.2** Strength ladder: run N seeded matches (N ≥ 500 hands per
  pairing) of Hard vs 3× Medium, Medium vs 3× Easy, and 4× of each. Record
  win rate, deal-in rate, average placement, average score, riichi rate,
  call rate, tenpai-at-draw rate. Expect Hard > Medium > Easy on average
  placement; record the numbers here.
- [ ] **4.3** Obvious-play checks (unit tests on fixed hands):
  - always takes a winning tsumo/ron with yaku (unless a documented
    reason, e.g. Hard declining a cheap ron as all-last first place);
  - never discards a tile that leaves tenpai for a worse shape when an
    equal-shanten better wait exists (Medium/Hard);
  - doesn't call into a no-yaku open hand it can't finish (yakuless pon);
  - Hard folds against riichi with a weak hand: discards genbutsu/suji
    first; doesn't push 1-shanten trash into a dealer riichi.
- [ ] **4.4** Riichi decisions: declares when tenpai and closed (at least
  at Medium/Hard), not on the last turn when it can't be afforded, not
  when below 1000 points.
- [ ] **4.5** Personality seed: `personality_seed` changes choices between
  seats, but a given seed is reproducible (saves and replays depend on
  this).
- [ ] **4.6** Pace: decision time per difficulty (see §6.1); no visible
  stall at `cpu_step_delay_ms = 0`.
- [ ] **4.7** Hidden information: AI decisions use only `PlayerView`
  (no peeking at other hands, the wall or ura dora). Check every field the
  AI reads.

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 5. Bugs and robustness

- [ ] **5.1** Triage every `unwrap`/`expect`/`panic!` from 1.4 that is
  reachable: replace with an error or a documented invariant (with a debug
  assertion and a test that the invariant holds).
- [ ] **5.2** Terminal states: start at 80×24, then shrink to 40×12, 1×1
  and back while on each screen (menu, setup, table, every overlay,
  popups, replay). No panic (ratatui `Rect` math, `u16` underflow), and the
  layout recovers.
- [ ] **5.3** Terminal restore: after quit, after a panic (force one in a
  debug build), after Ctrl+C / SIGTERM: raw mode off, cursor visible,
  alternate screen left.
- [ ] **5.4** Input: hold a key (repeat), mash keys during CPU turns and
  animations, press table keys while an overlay is open, `Esc` everywhere.
  No action applied twice, none applied to the wrong seat, no stale menu
  after the phase moved on.
- [ ] **5.5** Edge game states via scenarios: last tile with a pending kan;
  rinshan win; chankan + double ron; four riichis with one also winning;
  a player at exactly 0 and below 0; honba ≥ 10 (layout); a hand where
  all four seats are CPU (spectator).
- [ ] **5.6** Long run: leave a 4× CPU match at `cpu_step_delay_ms = 0`
  for an hour (or 100 matches looped). Memory stays flat, no panic.
- [ ] **5.7** Error paths are shown, not swallowed: missing config dir,
  unreadable config, invalid TOML, unknown keybind action, unwritable
  `recordings_dir`, disk full on save. The player sees a message; the log
  has the detail.

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 6. Performance

Measure with release builds; record machine, median of five runs.

- [ ] **6.1** AI decision time per difficulty: median and worst case over
  the §4.2 runs, per action type (discard, call, riichi). Target: worst
  case well under one frame budget at `cpu_step_delay_ms = 0`, or at least
  under 50 ms. Look at shanten and efficiency (`ai/shanten.rs`,
  `ai/efficiency.rs`): memoization, allocation per call.
- [ ] **6.2** Recommendations overlay (`rules/standard/recommendations/`):
  time to compute for the worst hands (14 tiles, many 1-shanten discards,
  open hands). It runs on the UI thread on keypress (check); must not
  freeze input.
- [ ] **6.3** Scoring and decomposition: `score_win` and pattern
  enumeration on hands with many readings (chinitsu, ryanpeikou shapes).
- [ ] **6.4** Engine throughput: matches per second at 4× Easy without UI
  (used by §2.5 and §4.2; also a sanity number for future online use).
- [ ] **6.5** Render loop: idle CPU use on the table screen (should be
  near zero when nothing animates and no timer runs); frame time with
  timers and animations; redraws per second.
- [ ] **6.6** Replay seek: time to seek to the last event of a long
  hanchan (does `RecordingPlayer` re-apply from 0 every seek?).
- [ ] **6.7** Startup: `rrmj --help`, time to first frame, time to list a
  `recordings_dir` with 1000 files (each is parsed in full to read
  `match_status`?).
- [ ] **6.8** Save file size for a full hanchan; event log growth.

| What | Result | Notes |
| ---- | ------ | ----- |
|      |        |       |

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 7. UI correctness and usability

- [ ] **7.1** Every screen at 80×24 and at a large size (e.g. 200×60):
  nothing clipped that matters (hand, melds, rivers of all four seats,
  dora, scores, timers, actions bar), nothing overlapping. Screenshots
  attached to findings.
- [ ] **7.2** Information shown matches the engine for one full match:
  scores, round/wind/honba/riichi sticks, dora indicators, wall count,
  each seat's wind, riichi markers in rivers, called-tile direction in
  melds, furiten indicator (if any; if none, should there be?).
- [ ] **7.3** Dora highlight (Phase 15.3): correct for aka, kan dora,
  doesn't clash with selection or the drawn-tile emphasis.
- [ ] **7.4** Both themes (`default`, `high-contrast`) on a dark and a
  light terminal background, and with `NO_COLOR`; colour is never the
  only signal (aka, dora, selection, whose turn).
- [ ] **7.5** Keybinds: the help overlay (`h`) lists exactly what works on
  each screen; custom `keybinds.toml` rebinding is honoured everywhere,
  including hints in the actions bar; duplicate keys across sections that
  are active at once (e.g. `y` is a hard-coded alternate for the rules
  overlay — does any user binding to `y` conflict?); unknown action error
  is readable.
- [ ] **7.6** First-time player walk-through: start with no config, play
  a hand without reading the docs. Note every point of confusion (how to
  discard, how to call, how to see the score, how to leave).
- [ ] **7.7** Rules overlay (`?`) and recommendations (`e`): content
  matches `RULES.md`; scrolling reaches the end; works at 80×24.
- [ ] **7.8** Settings screen: every option changes what it says, is
  written back on `Esc`, survives a restart, and keeps comments or
  unrelated keys in the user's `config.toml` (check what write-back does
  to a hand-edited file).
- [ ] **7.9** Leads from §1:
  - The new-game setup always shows `> Start match`, selected or not, and
    the field list wraps, so it's easy to press Enter on another row.
  - The hand renders as one run, e.g. `3p5pr7p9p2s3s5srEESN`: the aka
    marker `r` reads as either the end of one tile or the start of the
    next. Space the tiles or mark aka another way.
  - The setup popup is drawn over the main menu box at 100×32, cutting
    its borders.
- [ ] **7.10** Messages and wording: event text (`app/event_text.rs`),
  errors, prompts — consistent terms (one name per yaku, romaji spelled as
  in `RULES.md`), no debug formatting (`{:?}`) leaking to the player.

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 8. Security

rrmj is offline and has no network code, so the surface is the files it
reads and writes and its dependencies. Threat: a hostile or broken
scenario pack, save or config file (scenario packs are meant to be shared).

- [ ] **8.1** Dependencies: `cargo deny check` (all checks, not only
  `licenses`) and `cargo audit`. Lead: CI's `deny.yml` and PLAN §8 run
  only `cargo deny check licenses`, so advisories and bans are never
  checked. Add `advisories` and `bans` to CI.
- [ ] **8.2** Recording input (`MatchRecording::from_json` / `validate` /
  `restore`): try hostile files and check each gives an error, never a
  panic or hang:
  - seat indices ≥ 4, wrong number of players/hands, negative or huge
    scores, tile ids out of range, duplicate tiles, more than 4 of a kind,
    more than 4 aka;
  - `event_index` against `events` (validated) but also events that don't
    match the snapshot, or illegal events (`apply_until` on a forged log);
  - huge arrays (millions of events), deep nesting, a multi-GB file
    (`read_to_string` is unbounded: add a size cap);
  - `rules_config` with nonsense values (`starting_points` ≤ 0,
    `target_score` already reached, `triple_ron` without `double_ron`).
  Write each as a test in `librrmj/src/replay/tests.rs`.
- [ ] **8.3** Scenario listing reads every file in `scenarios_dir` and
  `recordings_dir`: one bad file must not break the list; symlinks, FIFOs
  (a read on a FIFO blocks the UI forever), directories named `*.json`,
  non-UTF-8 names.
- [ ] **8.4** Writes: saves, config write-back and scenario import use
  `fs::write` (truncate then write). A crash or full disk mid-write loses
  the old file. Write to a temp file and rename. Check the import path
  can't be made to overwrite files outside `scenarios_dir` (file names
  with `..` or `/` taken from the source).
- [ ] **8.5** Path handling: `~` expansion, `--config`/`--keybinds`
  pointing at a directory or a non-existent file, `XDG_*` unset or
  relative (XDG spec says ignore relative values).
- [ ] **8.6** Config and keybinds TOML: hostile values (huge timers,
  `u64` overflow into `Duration`/`Instant` addition — `Instant::now() +
  Duration::from_millis(u64::MAX)` panics; negative seat; unknown theme).
- [ ] **8.7** Logging: where the log file goes, its permissions, that it
  doesn't grow without bound, and that `RUST_LOG` misuse can't crash it.
- [ ] **8.8** Terminal output: text from files (scenario `meta.title`,
  `description`, file names) printed to the terminal is stripped of
  control/escape characters (a crafted title can otherwise inject
  terminal escape sequences).

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 9. Features against the plan

A matrix of what `docs/PLAN.md` and the README promise, against what the
code does. **✓** done, **part** partly (say which part), **—** missing.
Every gap gets a reason: **not done** (becomes a follow-up or moves to a
later phase), **deferred** (already in Phase 16/17), **dropped** (remove
from the docs).

- [ ] **9.1** Every checked box in PLAN §9 and §10: open the code (or the
  test) that implements it. Unchecked-but-done or checked-but-missing
  items are findings.
- [ ] **9.2** PLAN §1.3 "done" criteria for the standard profile, one by
  one, with evidence (test name or scenario file).
- [ ] **9.3** Phase 16 (TUI polish, open): decide for each of 16.1–16.5
  whether it's a release blocker or moves to post-release, and record it.
- [ ] **9.4** Every config key in `examples/config.toml` and every action
  in `examples/keybinds.toml` is read by the code, and every key the code
  reads is in the examples.
- [ ] **9.5** Feature flags: `ai`, `serde`, `test-util`, `debug-menu` —
  each builds alone, does what `ARCHITECTURE.md` §1.4 says, and the
  release build excludes `debug-menu`.

| Feature | `librrmj` | `rrmj` | Gap |
| ------- | :-------: | :----: | --- |
|         |           |        |     |

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 10. Docs

Read each doc as a new user (README, `examples/`) or a new contributor
(`docs/`), then check it against the code.

- [ ] **10.1** README: can a new user install, run, configure and find
  the rules from it alone? It has no install/build section, no controls
  summary and no link to `docs/RULES.md`.
- [ ] **10.2** `docs/RULES.md` against the engine (with §2): every rule
  the engine applies is written down, including the choices made in §2
  (kuikae, kan dora timing, karaten, bust, agari-yame, leftover sticks).
- [ ] **10.3** `docs/REPLAY.md` against `librrmj/src/replay/`: every field,
  `format_version`, migration notes.
- [ ] **10.4** `docs/DEBUG_SCENARIOS.md`: catalog equals the files
  (already enforced by a test), and the "how to add" steps work as written.
- [ ] **10.5** `docs/ARCHITECTURE.md` and `docs/PLAN.md` against the tree.
  Leads:
  - PLAN names files and modules that don't exist:
    `rules/standard/cheatsheet.rs`, `win_combinations/`,
    `librrmj/src/test_util/fixtures.rs`,
    `librrmj/tests/common/fixtures.rs`, `docs/TUI_BUGS.md`, and `rng/` in
    the §2 layout.
  - PLAN §11 lists `rand_chacha` and `proptest`; neither is a dependency.
  - PLAN §1.3 says Phases 15+ are out of scope until Phase 14 is
    complete; Phase 14 and 15 are done.
  - The recording type is `GameRecording`; PLAN and `REPLAY.md` call it
    `MatchRecording`.
  - ARCHITECTURE §8 lists `cargo deny check` and default-feature clippy;
    PLAN §8 lists `cargo deny check licenses` and no default clippy. Make
    them one list.
- [ ] **10.6** `examples/`: comments match behavior. Leads:
  - `examples/config.toml` says new rulesets come in "Phase 16.4"; that
    phase is theming, rulesets are 17.4.
  - The response-timer comment (see 3.5).
  - `examples/theme.toml` documents tokens for custom themes that aren't
    loaded; say so at its top, as the README does.
- [ ] **10.7** `cargo doc`: public items of `librrmj` documented; no
  broken intra-doc links (`RUSTDOCFLAGS="-D warnings"`); a crate-level
  example that plays one hand.
- [ ] **10.8** `CHANGELOG.md` ready for the first release entry.

| Finding | Severity | Status |
| ------- | -------- | ------ |
|         |          |        |

---

## 11. Wrap-up

- [ ] **11.1** Every finding is fixed, accepted with a reason, or a
  follow-up step in `docs/PLAN.md` §10.
- [ ] **11.2** No **high** finding is open. **medium** ones are fixed or
  explicitly accepted for the release.
- [ ] **11.3** PLAN §8 gates pass; `cargo deny check` (all checks) passes.
- [ ] **11.4** A summary paragraph at the top of this file: date, commit,
  what was found and fixed, what's left.
