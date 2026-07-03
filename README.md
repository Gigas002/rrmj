# rrmj

Rust riichi mahjong: **`librrmj`** (rules engine) and **`rrmj`** (terminal client).

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

## License

GPL-3.0-only — see [LICENSE](LICENSE).

## 3rd-party contents

Cheatsheet designs are based on [Riichi Cheat Sheet](https://drive.google.com/drive/folders/18hxO5DMVAqxSNV9VvpjAg6YjyPVAMzyS)
