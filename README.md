# rrmj

Rust riichi mahjong rules engine (`librrmj`) and terminal client (`rrmj`).

**v0.1** ships standard 4-player Japanese riichi: play a full hanchan locally against three CPU opponents at easy, medium, or hard difficulty.

## Requirements

- Rust toolchain (edition 2024; stable recommended)
- A terminal with Unicode support (plain-text tile labels such as `2m`, `7p`)

## Build and run

From the repository root:

```bash
cargo build --release
cargo run -p rrmj --release
```

Install the TUI binary:

```bash
cargo install --path rrmj
rrmj
```

Library only (no TUI):

```bash
cargo build -p librrmj --no-default-features
```

With all optional features (AI, serde, etc.):

```bash
cargo build --workspace --all-features
```

## Configuration

Config files live under `$XDG_CONFIG_HOME/rrmj` (usually `~/.config/rrmj`). Missing files fall back to built-in defaults.

| File | Purpose |
|------|---------|
| `config.toml` | Theme, default CPU difficulty, preferred seat |
| `keybinds.toml` | Hotkey map (`global`, `menu`, `table`, `overlay` sections) |

Override paths on the command line:

```bash
rrmj --config /path/to/config.toml --keybinds /path/to/keybinds.toml
```

Annotated examples with every option explained: [examples/config.toml](examples/config.toml), [examples/keybinds.toml](examples/keybinds.toml), and [examples/theme.toml](examples/theme.toml) (palette token reference).

### Example `config.toml`

```toml
theme = "default"              # or "high-contrast"
default_difficulty = "medium"  # easy | medium | hard
human_seat = 0                 # 0=East … 3=North
```

Settings changed in the in-game **Settings** screen are written to `config.toml` when you press Esc.

Tiles are shown as plain text labels (`2m`, `7p`, `Es`, …) with theme colors.

### CPU difficulty

| Tier | Behavior |
|------|----------|
| **Easy** | Random legal moves; takes obvious wins |
| **Medium** | Shanten-based discards; basic calls and defense |
| **Hard** | Better tile efficiency and defensive play vs riichi |

Per-seat difficulty is chosen in the new-game setup screen.

## In-game overlays

| Key | Overlay |
|-----|---------|
| `h` | Full keybind list |
| `?` or `y` | Rules and yaku reference (aligned with [docs/RULES.md](docs/RULES.md)) |

## Rules

The engine implements the **standard** profile documented in [docs/RULES.md](docs/RULES.md). Scoring and yaku logic live in `librrmj` behind the `RulesProfile` trait — the TUI does not duplicate rules.

## Crates

| Crate | Role |
|-------|------|
| `librrmj` | Tiles, state machine, scoring, AI, replay API |
| `rrmj` | ratatui client — presentation and input only |

## Development

Quality gates (see [docs/PLAN.md](docs/PLAN.md)):

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
typos
cargo deny check
```

## License

GPL-3.0-only — see `LICENSE` if present in the repository.
