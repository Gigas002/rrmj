# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-06-08

First release.

### librrmj

- Standard Japanese riichi rules profile (yaku, fu, payments, dora, riichi, exhaustive and abortive draws).
- Event-sourced match API with `Match::step()` / `apply_action()` and in-memory `Replay`.
- CPU opponents: easy, medium, and hard (`ai` feature).
- Deterministic wall and AI via seeded RNG.

### rrmj-tui

- Main menu, new-game setup (human/CPU per seat), and full table play.
- Legal-action-only hotkeys with customizable `keybinds.toml`.
- `config.toml` for theme, defaults, ASCII tile mode, and animations.
- Built-in **default** and **high-contrast** themes.
- ASCII rendering (default) with optional motion effects; Unicode bracket mode.
- Keybind help (`h`) and rules/yaku reference overlay (`?` / `y`).
- Hand result and match summary screens.

### Documentation

- [docs/PLAN.md](docs/PLAN.md) — architecture and phased roadmap.
- [docs/RULES.md](docs/RULES.md) — standard rules profile reference.

[0.1.0]: https://github.com/Gigas002/rrmj/releases/tag/v0.1.0
