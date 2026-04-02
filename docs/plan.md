# Plan: Friction Log MVP

> Source PRD: [docs/prd.md](prd.md)
> User Journeys: [docs/user journeys/](user%20journeys/)
> Jobs-to-be-Done: [docs/jobs-to-be-done.md](jobs-to-be-done.md)

## Architectural decisions

Durable decisions that apply across all phases:

- **Platform**: Tauri v2 desktop app (macOS primary, Windows/Linux secondary)
- **Backend**: Rust (async via Tokio, included with Tauri)
- **Frontend**: Vue 3 + TypeScript, Vite bundler
- **Database**: SQLite via `rusqlite` (bundled), single local file
- **Package manager**: Bun
- **App identifier**: `dev.frlog.app`

### Schema (logical entities)

| Table | Purpose |
|---|---|
| `manual_logs` | User-submitted friction notes with timestamp, raw text, parsed command, inferred tags, confidence, related app context |
| `focus_events` | Passive desktop context: timestamp, app_name, window_title, normalized_title, redaction_status |
| `waiting_periods` | Waiting/blocking state markers: start_time, end_time, note, direction (waiting vs blocking) |
| `derived_sessions` | Computed app sequences with duration and switch count |
| `friction_clusters` | Grouped friction patterns: title, summary, category, confidence, status (ignore/watch/worth_fixing) |
| `automation_briefs` | Generated briefs: source cluster, generated text, exported_at, resolution_status |
| `privacy_rules` | Per-app privacy settings: app_name, mode, exclusion_status |
| `user_feedback` | Corrected tags, ignored nudges, usefulness ratings |

### Three processing paths

1. **Capture path** — must be instant. Write note + context, no blocking analysis.
2. **Analysis path** — background. Clustering, tagging, session derivation, digest generation.
3. **Output path** — on-demand. Review UI, brief generation, export.

### Friction categories

- Repetitive Work
- Coordination Tax
- Context-Switching Cost
- Bottleneck
- Guilt Pile
- (Derived) Ambiguity, Maintenance Debt, Emotional Drag

### Routes / Views

| Route | Purpose |
|---|---|
| Capture overlay | Global-shortcut-triggered spotlight input (separate Tauri window) |
| `/` | Dashboard / home — entry point to review |
| `/logs` | Chronological list of captured friction logs |
| `/review` | Weekly review with top clusters and triage |
| `/cluster/:id` | Cluster detail with examples, evidence, and brief generation |
| `/brief/:id` | Automation brief view/edit |
| `/settings` | Privacy controls, app exclusions, data management |
| `/onboarding` | First-run privacy explanation and configuration |

---

## Phase 1: Quick Capture

**User stories**:
- As a user, I want to log a frustration in one sentence so I can capture it without breaking flow.
- As a user, I want the quick-capture window to disappear immediately after submission.

**JTBD**: 1 (Capture friction in the moment)

### What to build

A complete capture loop: the user presses a global keyboard shortcut, a lightweight spotlight-style overlay window appears, they type a freeform note, press Enter, and the window closes instantly. The note is stored in SQLite with a timestamp. A basic logs list view shows all captured notes in reverse chronological order so the user can verify their entries.

This phase establishes the foundational infrastructure: SQLite database setup, Tauri command layer, Vue routing, and the two-window architecture (main window + capture overlay).

### Acceptance criteria

- [x] SQLite database is created on first launch with `manual_logs` table
- [x] Global shortcut (e.g. `CmdOrCtrl+Shift+L`) opens a borderless, centered overlay window
- [x] Overlay has a single focused text input — no other UI chrome
- [x] Pressing Enter submits the note and closes the overlay immediately
- [x] Pressing Escape closes the overlay without submitting
- [x] Note is stored with timestamp in `manual_logs`
- [x] Backend processing (any future analysis) happens async after window closes
- [x] Main app window has a `/logs` view showing all entries in reverse chronological order
- [x] Each log entry displays timestamp and raw text
- [x] Capture-to-close latency target: under 150ms after Enter

---

## Phase 2: Passive Context + Privacy Foundations

**User stories**:
- As a user, I want app context to be attached automatically so I do not need to fill in metadata.
- As a user, I want to exclude sensitive apps from tracking.
- As a user, I want to pause tracking at any time.
- As a user, I want to know exactly what is being collected and what is not.

**JTBD**: 5 (Protect privacy while still getting value)

### What to build

A background Rust task that polls the active focused application every 2-5 seconds and records changes to the `focus_events` table. When a manual log is submitted, the current app context (and recent app sequence) is automatically attached. Privacy controls allow the user to: pause/resume all tracking, maintain an app exclusion list, choose between app-name-only and app+title modes, and view/delete collected data. A first-run onboarding screen explains what is collected in plain language and lets the user configure settings before tracking begins.

### Acceptance criteria

- [x] Background task polls active window at configurable interval (default 3s)
- [x] Only focus changes are persisted (no duplicate consecutive entries)
- [x] `focus_events` table stores timestamp, app_name, and optional window_title
- [x] Manual logs are enriched with the active app at time of capture
- [x] Settings view exists at `/settings` with:
  - [x] Pause/resume tracking toggle
  - [x] App exclusion list (add/remove apps)
  - [x] App-only vs app+title mode toggle
  - [x] View collected data section
  - [x] Delete all data button with confirmation
- [x] Default is app-name-only (title capture requires explicit opt-in)
- [x] Excluded apps produce no focus events
- [x] Onboarding screen shown on first launch before tracking starts
- [x] Onboarding explains: what is collected, what is not (no screenshots, no keystrokes, no clipboard)
- [x] Suggested exclusions offered for password managers, banking apps, messaging apps
- [x] Background polling does not noticeably impact CPU
- [x] Tracking recovers gracefully after sleep/resume

---

## Phase 3: Dot Commands + Waiting States

**User stories**:
- As a user, I want low-friction state markers (`.w`, `.b`, `.r`, `.c`, `.g`)
- As a user, I want waiting periods to be visible in review mode

**JTBD**: 7 (Track coordination overhead), 8 (Track blocked work), 9 (Track guilt-pile tasks)

### What to build

The capture input parser recognizes dot-command prefixes and stores parsed commands alongside the raw text. `.w` and `.b` commands create entries in the `waiting_periods` table with start times; subsequent entries or explicit toggling infer end times and durations. All dot commands (`.r`, `.c`, `.g`) attach the corresponding friction category tag. The logs list view shows command badges and waiting/blocking state indicators. A waiting-period summary is visible showing active and resolved waiting states with durations.

### Acceptance criteria

- [x] Input starting with `.w ` is parsed as a waiting-state marker
- [x] Input starting with `.b ` is parsed as a blocking-state marker
- [x] Input starting with `.r ` is tagged as Repetitive Work
- [x] Input starting with `.c ` is tagged as Coordination Tax
- [x] Input starting with `.g ` is tagged as Guilt Pile
- [x] `waiting_periods` table stores start_time, end_time, note, direction
- [x] Waiting/blocking states can be ended by a new `.w`/`.b` entry or explicit toggle
- [x] Duration is computed for completed waiting periods
- [x] Log entries display parsed command badge (e.g. "waiting", "blocking", "repetitive")
- [x] Logs view can be filtered by command type / category
- [x] Raw text after the dot command is preserved as the note content

---

## Phase 4: Pattern Detection + Clustering

**User stories**:
- As a user, I want the app to notice recurring friction patterns so I can see what is worth fixing.
- As a user, I want the system to ask clarifying questions only when useful.
- As a user, I want to distinguish between real friction and legitimate work patterns.

**JTBD**: 2 (Discover patterns I would not notice manually), 10 (Separate necessary work from bad design)

### What to build

A background analysis pipeline that processes manual logs and focus events to identify patterns. Notes are auto-tagged with friction categories using keyword/heuristic matching (with LLM enhancement as a future option). Semantically similar logs are grouped into `friction_clusters` with generated summaries, category assignments, and confidence scores. Each cluster links back to contributing log entries and focus events as evidence. The cluster view shows grouped patterns with examples and app context that contributed to the grouping.

### Acceptance criteria

- [x] Manual logs are auto-tagged with friction categories on submission
- [x] Tags include a confidence score (0.0–1.0)
- [x] Users can edit or correct inferred tags
- [x] Similar logs are grouped into clusters (by keyword similarity, category, and app context)
- [x] Each cluster has: title, summary, category, confidence, list of contributing logs
- [x] Cluster summaries are generated from contributing evidence
- [x] App context (frequent app sequences) contributes to clustering
- [x] Clusters are recomputed periodically or on-demand
- [x] `/cluster/:id` view shows cluster detail with all contributing examples
- [x] Confidence score is visible on each cluster
- [x] User corrections feed back into future tagging accuracy

---

## Phase 5: Weekly Review + Triage

**User stories**:
- As a user, I want a weekly review of my top friction clusters so I can prioritize what to automate.
- As a user, I want to mark a cluster as ignore, watch, or worth fixing.

**JTBD**: 3 (Prioritize what is worth fixing)

### What to build

A review surface that presents a weekly summary of the user's friction. The review view shows the top 3 clusters ranked by frequency and estimated time cost, with each cluster showing contributing examples, app context patterns, and category. Users triage each cluster by marking it Ignore (suppress from future reviews), Watch (keep monitoring), or Worth Fixing (queue for brief generation). The review also includes aggregate stats: total logs this week, top friction categories, waiting time totals. Users can rename clusters and correct category assignments.

### Acceptance criteria

- [x] `/review` view shows weekly summary with date range
- [x] Top 3 friction clusters displayed ranked by frequency and estimated cost
- [x] Each cluster card shows: title, category, count, estimated time, top examples
- [x] Triage controls on each cluster: Ignore / Watch / Worth Fixing
- [x] Ignored clusters are suppressed from future reviews (but accessible via filter)
- [x] "Worth Fixing" clusters are highlighted and available for brief generation
- [x] Aggregate stats section: total logs, top categories, total waiting time
- [x] Users can rename cluster titles
- [x] Users can correct cluster categories
- [x] Weekly review defaults as primary cadence (daily summary available optionally)
- [x] Non-judgmental, supportive tone in all review copy (per PRD tone guidelines)

---

## Phase 6: Automation Brief + Export

**User stories**:
- As a user, I want a structured automation brief generated from my logs.
- As a user, I want to copy or export that brief into an AI tool or automation platform.
- As a user, I want to see which examples and signals informed the brief.

**JTBD**: 4 (Generate an automation-ready artifact)

### What to build

From any "Worth Fixing" cluster, the user can generate a structured automation brief. The brief follows the PRD template: Problem, Trigger, Current workflow, Apps involved, Frequency, Estimated time cost, Emotional cost, Dependencies/blockers, Example instances, Desired outcome, Constraints, Candidate automation approaches, and Agent-ready prompt/spec. The brief is generated from cluster evidence (logs, app context, waiting periods) and stored in `automation_briefs`. The user can edit the brief before exporting. Export options: copy as plain text, copy as Markdown, copy as agent spec.

### Acceptance criteria

- [x] "Generate Brief" action available on Worth Fixing clusters
- [x] Brief includes all required sections from the PRD template
- [x] Brief references actual log examples and app context as evidence
- [x] Generated brief is stored in `automation_briefs` table linked to source cluster
- [x] `/brief/:id` view displays the brief in a readable, editable format
- [x] User can edit any section of the brief before exporting
- [x] "Copy as Plain Text" button copies brief to clipboard
- [x] "Copy as Markdown" button copies formatted Markdown to clipboard
- [x] "Copy as Agent Spec" button copies the agent-ready prompt section
- [x] Export action records `exported_at` timestamp
- [x] Source examples are visible alongside the brief for reference

---

## Phase 7: Outcome Tracking + Closed Loop

**User stories**:
- As a user, I want to mark a friction cluster as resolved so I can measure whether the product actually helped.
- As a user, I want the app to ask later whether the automation reduced the friction.

**JTBD**: 6 (Close the loop on improvements)

### What to build

After a brief has been generated and exported, the system tracks outcomes. Users can mark a cluster/brief as Resolved, Reduced, or Unchanged at any time. The app prompts for outcome feedback 7-14 days after export. Users can optionally estimate recovered time per week. A "wins" history view shows resolved clusters, estimated time savings, and improvement timeline. Outcome data feeds back into future cluster prioritization (resolved patterns boost product credibility, unchanged patterns inform better recommendations).

### Acceptance criteria

- [x] Cluster/brief status options: Resolved / Reduced / Unchanged
- [x] Status can be updated from cluster detail or brief detail views
- [x] Follow-up prompt appears 7-14 days after brief export (non-intrusive, dismissible)
- [x] Follow-up includes one-click outcome selection with optional free-text note
- [x] Optional "estimated time saved per week" input (rough estimate, can skip)
- [x] Outcome stored in `automation_briefs` table (resolution_status, resolved_at, estimated_savings)
- [x] Wins history view shows all resolved/reduced clusters with:
  - [x] Timeline of improvements
  - [x] Cumulative estimated time recovered
  - [x] Which briefs led to action
- [x] Unchanged outcomes are visible but framed constructively ("worth revisiting?")
- [x] Outcome data influences cluster ranking in future reviews (resolved patterns = product working)
