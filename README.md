# Friction Log

A desktop app that helps you discover what in your work is worth automating.

Log moments of friction, spot patterns, and generate structured briefs you can hand off to an AI agent, a script, or a teammate.

**Core loop:** Log friction → See patterns → Get brief → Hand off → Reclaim time

## Why

Most people know their work has inefficiencies but can't pinpoint exactly where time goes. Existing tools focus on time tracking or habit tracking — they increase reflection but rarely produce something actionable.

Friction Log starts from the lived moments of annoyance, waiting, repetition, and interruption that occur during real work. It turns those moments into structured automation briefs — the core artifact that makes the difference between noticing a problem and actually fixing it.

## What it does

- **Quick capture** — Global shortcut (`Cmd+Shift+L`) opens a spotlight-style input. Type a one-liner, press Enter, and you're back to work in under a second.
- **Dot commands** — Prefix notes with `.w` (waiting), `.b` (blocking others), `.r` (repetitive), `.c` (coordination overhead), or `.g` (guilt pile) for instant categorization.
- **Passive context** — Tracks which app you're using when you log (app name only by default, window titles opt-in). Excluded apps are never recorded.
- **Auto-tagging** — Plain text notes are automatically categorized into friction types (repetitive work, bottleneck, coordination tax, context switching, guilt pile) with confidence scores.
- **Pattern detection** — Clusters similar logs by category and keyword similarity. Surfaces recurring patterns you might not notice on your own.
- **Weekly review** — Summary of your top friction clusters with aggregate stats. Triage each pattern as Ignore, Watch, or Worth Fixing.
- **Automation briefs** — Generates a structured 13-section brief from a cluster's evidence: problem, trigger, current workflow, apps involved, frequency, time cost, emotional cost, examples, desired outcome, constraints, candidate approaches, and an agent-ready spec.
- **Export** — Copy briefs as plain text, Markdown, or agent spec directly to clipboard.
- **Outcome tracking** — Mark briefs as Resolved, Reduced, or Unchanged. Track estimated time saved. A wins history shows cumulative impact.

## Privacy

Privacy is a core product feature, not an afterthought.

- All data stays on your machine (local SQLite database)
- No screenshots, no keystroke logging, no clipboard scraping, no content scraping
- App name only by default — window title capture is opt-in
- Exclude sensitive apps (password managers, banking, messaging) from tracking entirely
- Pause/resume tracking at any time
- View and delete all collected data from Settings

## Tech stack

- **Desktop shell:** [Tauri v2](https://v2.tauri.app)
- **Backend:** Rust
- **Frontend:** Vue 3 + TypeScript
- **Database:** SQLite (via rusqlite)
- **Build tooling:** Vite, Bun

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Bun](https://bun.sh/) (or Node.js)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) prerequisites for your platform
- (Optional) [Task](https://taskfile.dev/) for the task runner

### Install dependencies

```sh
bun install
```

### Development

```sh
# Run in dev mode with hot reload
task dev
# or without Task:
bun run tauri dev
```

### Build

```sh
# Production build (outputs .app and .dmg on macOS)
task build
# or:
bun run tauri build
```

### Test

```sh
# Run Rust tests
task test
# or:
cd src-tauri && cargo test
```

### Other commands

```sh
task fmt       # Format Rust code
task check     # Type-check frontend + Rust
task clean     # Clean build artifacts
```

## Usage

1. **First launch** — Complete the privacy onboarding (choose what to track, exclude sensitive apps).
2. **Capture friction** — Press `Cmd+Shift+L` anytime something feels annoying, repetitive, or blocking. Type a short note and press Enter.
3. **Use dot commands** — `.w waiting on API approval`, `.r pasted ticket IDs again`, `.g still haven't cleaned up docs`.
4. **Review patterns** — Click "Patterns" tab, then "Recompute patterns" to cluster your logs. Click "Review" for a weekly summary.
5. **Triage** — Mark clusters as Worth Fixing to queue them for brief generation.
6. **Generate brief** — Open a cluster and click "Generate Automation Brief". Edit any section, then copy as plain text, Markdown, or agent spec.
7. **Track outcomes** — After acting on a brief, mark it as Resolved, Reduced, or Unchanged. Check "Wins" to see your cumulative impact.

## Project structure

```
src/                    # Vue 3 frontend
  views/                # Page components (Logs, Capture, Review, Settings, etc.)
  router.ts             # Vue Router configuration
  App.vue               # Root component
src-tauri/              # Rust backend
  src/
    lib.rs              # App setup, plugin registration, global shortcut
    db.rs               # SQLite schema, migrations, queries
    commands.rs          # Tauri command handlers
    parser.rs            # Dot command parser
    analyzer.rs          # Auto-tagger and clustering engine
    brief.rs             # Automation brief generator
    tracker.rs           # Background focus tracking
docs/                   # Product documentation (PRD, user journeys, plan)
```

## License

MIT
