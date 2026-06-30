# rrmj

Rust riichi mahjong: **`librrmj`** (rules engine) and **`rrmj`** (terminal client).

The project is under active development. Workspace crates are version `0.1.0` in source, but **nothing has been published or tagged as a release yet**.

## What works today

**Engine (`librrmj`)**

- Event-sourced match flow: wall, turns, calls, riichi, dora, wins, honba, match end.
- **Standard** rules profile (`rules/standard/`) behind a `RulesProfile` boundary.
- Deterministic play from a seed plus an append-only `Action` / `Event` log.
- CPU opponents (easy / medium / hard) behind the `ai` feature.
- JSON recordings (`*.rrmj.json`) for saves, replays, and scenarios — see [docs/REPLAY.md](docs/REPLAY.md).

Rule coverage is broad and heavily tested, but the standard profile is **not** considered finished until every cheatsheet row is implemented, fu/limit-hand scoring is complete, and [docs/RULES.md](docs/RULES.md) matches the engine. See [docs/PLAN.md](docs/PLAN.md) for the backlog.

**Terminal client (`rrmj`)**

- Main menu: new game, load in-progress save, replays, scenarios, settings.
- Full table play vs three CPUs; hotkeys map only to **legal** actions from the engine.
- Pause menu: resume, export save, return to main menu, quit.
- Overlays: keybind help (`h`), rules reference (`?`), scores (`s`), win-path recommendations (`e`).
- Replay viewer for finished recordings.
- Themes: `default` and `high-contrast` (colored text tile labels such as `2m`, `7p`, `5pr`).
- Config and keybind files under `$XDG_CONFIG_HOME/rrmj` (see below).

The TUI does not implement rules on its own — it builds `Game` from `librrmj` and calls `apply_action`.

## Requirements

- Rust toolchain (edition 2024; stable recommended)
- A Unicode-capable terminal

## Build and run

From the repository root:

```bash
cargo run -p rrmj --release
```

Install the binary locally:

```bash
cargo install --path rrmj
rrmj
```

Library only (no TUI, no AI):

```bash
cargo build -p librrmj --no-default-features
```

With all optional features:

```bash
cargo build --workspace --all-features
```

Debug scenarios menu (not in default release builds):

```bash
cargo run -p rrmj --features debug-menu
```

## Configuration

Defaults live in `$XDG_CONFIG_HOME/rrmj` (usually `~/.config/rrmj`). Override paths at launch:

```bash
rrmj --config /path/to/config.toml --keybinds /path/to/keybinds.toml
```

| File | Purpose |
|------|---------|
| `config.toml` | Theme, rules profile, CPU difficulty, seat, timers, optional data directories |
| `keybinds.toml` | Hotkey map (`global`, `menu`, `table`, `overlay` sections) |

Annotated examples: [examples/config.toml](examples/config.toml), [examples/keybinds.toml](examples/keybinds.toml). [examples/theme.toml](examples/theme.toml) documents palette tokens for the built-in themes; custom theme files are not loaded yet.

In-game **Settings** writes changes back to `config.toml` when you press Esc.

**Data directories** (defaults under `$XDG_DATA_HOME/rrmj/`):

| Directory | Purpose |
|-----------|---------|
| `recordings/` | In-progress saves and finished replays (`match_status` filters which menu lists them) |
| `scenarios/` | User scenario packs (import from the Scenarios menu or copy JSON here) |

Repo fixtures for CI and debug builds: [examples/scenarios/](examples/scenarios/).

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/PLAN.md](docs/PLAN.md) | Roadmap and phase status |
| [docs/RULES.md](docs/RULES.md) | Standard rules reference |
| [docs/REPLAY.md](docs/REPLAY.md) | Recording file format |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate layout, settings flow, quality gates |
| [docs/DEBUG_SCENARIOS.md](docs/DEBUG_SCENARIOS.md) | Debug-menu scenarios (`debug-menu` feature) |

## Crates

| Crate | Role |
|-------|------|
| `librrmj` | Tiles, state machine, scoring, AI, replay API |
| `rrmj` | ratatui client — presentation and input only |

## Development

Quality gates before commit (full list in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) §8):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-default-features
cargo test --workspace
cargo test --workspace --all-features
cargo doc --workspace --no-deps
typos
cargo deny check
```

## License

GPL-3.0-only — see [LICENSE](LICENSE).

## 3rd-party contents

Cheatsheet designs are based on [Riichi Cheat Sheet](https://drive.google.com/drive/folders/18hxO5DMVAqxSNV9VvpjAg6YjyPVAMzyS)
