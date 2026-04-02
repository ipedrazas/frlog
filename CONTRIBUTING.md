# Contributing to Friction Log

Thanks for your interest in contributing. This document covers how to set up the project, the conventions we follow, and how to submit changes.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Bun](https://bun.sh/) (or Node.js 18+)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform
- [Task](https://taskfile.dev/) (optional, for the task runner)

On macOS, you may need to grant accessibility permissions for active window detection.

## Setup

```sh
git clone https://github.com/ipedrazas/frlog.git
cd frlog
bun install
task dev   # or: bun run tauri dev
```

## Project structure

```
src/                    Vue 3 + TypeScript frontend
  views/                Page components
  router.ts             Route definitions
src-tauri/              Rust backend (Tauri v2)
  src/
    lib.rs              App setup, plugins, global shortcut
    db.rs               SQLite schema, migrations, queries
    commands.rs          Tauri command handlers
    parser.rs            Dot command parser
    analyzer.rs          Auto-tagger and clustering engine
    brief.rs             Automation brief generator
    tracker.rs           Background focus tracking
docs/                   PRD, user journeys, implementation plan
```

## Common commands

| Command | What it does |
|---|---|
| `task dev` | Run the app in dev mode with hot reload |
| `task build` | Production build (outputs .app/.dmg on macOS) |
| `task test` | Run Rust unit tests |
| `task check` | Type-check frontend (vue-tsc) and Rust (cargo check) |
| `task fmt` | Format Rust code (cargo fmt) |
| `task lint` | Run Rust linter (cargo clippy) |
| `task clean` | Remove build artifacts |

## Development workflow

1. Create a branch from `main` for your change.
2. Make your changes. Run `task check` and `task test` before committing.
3. Keep commits focused — one logical change per commit.
4. Open a pull request with a clear description of what changed and why.

## Code conventions

### Rust

- Run `cargo fmt` before committing. The CI will reject unformatted code.
- Run `cargo clippy` and address warnings.
- Keep Tauri commands thin — put logic in dedicated modules (`db.rs`, `analyzer.rs`, `brief.rs`), not in `commands.rs`.
- New database tables or columns need a migration in `db::init()` so existing databases upgrade cleanly.
- Unit tests go in the same file as the code they test, in a `#[cfg(test)] mod tests` block.

### TypeScript / Vue

- Run `vue-tsc --noEmit` to type-check. No unused variables or imports.
- Components live in `src/views/` and are wired through `src/router.ts`.
- Use `invoke()` from `@tauri-apps/api/core` to call Rust commands. Keep frontend logic minimal — the backend is the source of truth.
- Styles are scoped to each component. No global CSS beyond `App.vue` resets.

### General

- No unnecessary abstractions. Three similar lines are better than a premature helper.
- Don't add features beyond what was asked for in the issue or PR description.
- Privacy is a core concern. Never collect more data than necessary. Any new data collection must be opt-in and clearly explained to the user.
- The app should never shame the user. Review all user-facing copy for supportive, systems-oriented tone.

## Database changes

The app uses SQLite via `rusqlite`. The schema is defined in `db::init()`.

When adding a new table, add the `CREATE TABLE IF NOT EXISTS` statement to the `execute_batch` call in `init()`.

When adding a column to an existing table, add a migration check at the end of `init()`:

```rust
if !has_col("table_name", "new_column") {
    conn.execute_batch("ALTER TABLE table_name ADD COLUMN new_column TYPE DEFAULT value;")?;
}
```

This ensures existing databases upgrade without data loss.

## Adding a new Tauri command

1. Add the function to `commands.rs` with the `#[tauri::command]` attribute.
2. Register it in `lib.rs` inside the `generate_handler![]` macro.
3. Call it from the frontend with `invoke("command_name", { args })`.

## Tests

Rust tests live alongside the code:

```sh
task test
# or: cd src-tauri && cargo test
```

There are currently tests for:
- `parser.rs` — Dot command parsing (7 tests)
- `analyzer.rs` — Category inference and clustering (8 tests)

When adding new logic (especially in `parser.rs`, `analyzer.rs`, or `brief.rs`), add unit tests.

## Reporting issues

Open an issue on GitHub with:
- What you expected to happen
- What actually happened
- Steps to reproduce
- Your OS and version

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
