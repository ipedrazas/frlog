# PRD: Friction Log

Version: Draft v1
Author: Ivan
Product Type: Desktop application
Platform: Tauri v2 desktop app
Primary Audience: Knowledge workers, engineers, operators, managers, founders, and anyone using AI agents to automate repetitive work

## 1. Overview

Friction Log is a desktop application that helps users discover what in their work is worth automating, delegating, or redesigning.

It combines:
- lightweight manual logging of work friction
- passive desktop context signals
- pattern detection
- structured output in the form of automation briefs

The core value is not time tracking for its own sake. The product exists to turn vague frustration into a tangible, reusable artifact that can be handed off to an AI agent, teammate, or future self.

Core loop:

Log friction → See pattern → Get brief → Hand off to AI agent → Reclaim time

Primary product promise:

Find what to automate before you try to automate it.

## 2. Problem Statement

Many people feel overwhelmed by repeated busywork, coordination overhead, and context switching, but they lack a system for identifying:
- what repeats most often
- what drains attention
- what delays progress
- what is worth fixing
- what can realistically be automated

Existing tools often fail because they are optimized for:
- time tracking
- habit tracking
- journaling
- productivity dashboards

These tools may increase reflection, but they rarely produce a concrete artifact that leads to action.

Users need a lightweight way to capture moments of friction and a system that turns those moments into structured opportunities for automation.

## 3. Vision

Friction Log is a friction journal that pays out.

It helps users observe the drag in their day, identify patterns, and generate actionable briefs for AI agents and automation tools.

Rather than asking users to map their workflows in advance, Friction Log starts from the lived moments of annoyance, waiting, repetition, and interruption that occur during real work.

The product should feel:
- fast
- private
- non-judgmental
- useful within days, not weeks
- more like a work companion than a dashboard

## 4. Goals

### Primary goals
- Help users identify recurring friction in their work
- Convert friction into structured automation briefs
- Make it easy to hand those briefs to AI agents or automation tools
- Build a repeatable habit of noticing and logging high-value friction

### Secondary goals
- Detect patterns from passive desktop context
- Reduce the effort required to capture friction
- Help users estimate the recoverable value of fixing a workflow
- Build trust through privacy-first product design

### Non-goals
- Employee monitoring
- Surveillance or productivity scoring
- Precise timesheets or utilization tracking
- Keystroke logging
- Screen recording
- Full workflow automation execution inside the MVP

## 5. Target Users

### Primary users
- Engineers
- Product managers
- Operations staff
- Founders
- Analysts
- Researchers
- Individual contributors doing cross-tool digital work

### Ideal user characteristics
- Works across many desktop apps and browser tabs
- Has recurring tasks that feel annoying, repetitive, or interruptive
- Is interested in AI agents, automation, or delegation
- Wants concrete time savings, not just self-awareness

### Example user motivations
- “I know I waste time, but I don’t know on what exactly.”
- “I keep doing the same boring steps every week.”
- “So much of my day is chasing updates and approvals.”
- “I want to automate more, but I don’t know where to start.”
- “I need a prompt/spec I can hand to an AI tool.”

## 6. Core Product Thesis

The highest-value work signals are not full activity timelines. They are moments of friction.

These include:
- repeated tasks
- waiting on others
- blocking others
- context switching
- rework
- coordination overhead
- guilt-inducing tasks that are never prioritized

If these moments are captured lightly and clustered effectively, they can produce structured outputs that are worth acting on.

The automation brief is the core artifact of the product.

## 7. Key Concepts and Taxonomy

Friction Log will organize signals into psychologically resonant categories.

### Core friction categories
- Repetitive Work
  - recurring manual tasks
  - “Groundhog Day” workflows
  - repetitive but not difficult tasks
- Coordination Tax
  - chasing updates
  - waiting for approvals
  - gathering missing information
  - follow-up loops
- Context-Switching Cost
  - frequent switching between apps/tasks
  - interruptions to deep work
  - fragmented attention
- Bottleneck
  - work that blocks downstream progress
  - dependencies where one slow step stalls everything
  - waiting on others or others waiting on you
- Guilt Pile
  - recurring important-but-avoided tasks
  - hygiene/maintenance/admin work
  - “I should do this consistently but never get to it”

### Additional derived categories
- Ambiguity
  - unclear next step
  - uncertainty causing stalled action
- Maintenance Debt
  - recurring cleanup, admin, or upkeep
- Emotional Drag
  - dread, avoidance, low-grade friction with psychological weight

Note:
The system should infer categories by default. Users should not be forced to manually choose one every time.

## 8. Product Principles

- Capture friction, not everything
- Manual input should take under 5 seconds
- Passive signals should support, not replace, user intent
- The app must feel private and trustworthy
- Outputs must be action-oriented, not just reflective
- Real-time nudges should be rare and high-confidence
- The tone must be supportive and systems-oriented, not blaming
- Local-first should be the default

## 9. User Stories

### Capture
- As a user, I want to log a frustration in one sentence so I can capture it without breaking flow.
- As a user, I want the quick-capture window to disappear immediately after submission.
- As a user, I want app context to be attached automatically so I do not need to fill in metadata.

### Pattern detection
- As a user, I want the app to notice recurring friction patterns so I can see what is worth fixing.
- As a user, I want the system to ask clarifying questions only when useful.
- As a user, I want to distinguish between real friction and legitimate work patterns like research.

### Review
- As a user, I want a weekly review of my top friction clusters so I can prioritize what to automate.
- As a user, I want to mark a cluster as ignore, watch, or worth fixing.

### Output
- As a user, I want a structured automation brief generated from my logs.
- As a user, I want to copy or export that brief into an AI tool or automation platform.
- As a user, I want to see which examples and signals informed the brief.

### Privacy
- As a user, I want to exclude sensitive apps from tracking.
- As a user, I want to pause tracking at any time.
- As a user, I want to know exactly what is being collected and what is not.

### Follow-through
- As a user, I want to mark a friction cluster as resolved so I can measure whether the product actually helped.
- As a user, I want the app to ask later whether the automation reduced the friction.

## 10. Core User Flow

### Capture mode
1. User hits global shortcut
2. Spotlight-style input appears, centered, undecorated, fast
3. User types a short note in natural language
4. User presses Enter
5. Window disappears immediately
6. Backend processes note asynchronously
7. App stores note with timestamp and passive context

Example inputs:
- “Tired of copy-pasting Jira IDs into Slack.”
- “Spent 20m chasing approval for release.”
- “Rewriting the same status update again.”
- `.w waiting on design sign-off`
- `.b QA is waiting on me for logs`
- `.g never get around to weekly clean-up`

### Review mode
1. User opens review screen daily or weekly
2. App shows clusters and summaries
3. User reviews top friction areas
4. User marks clusters:
   - Ignore
   - Watch
   - Worth fixing
5. User generates automation brief
6. User copies/export brief to AI tool
7. User later marks outcome:
   - Resolved
   - Reduced
   - Unchanged

## 11. Product Scope

## MVP Features

### A. Quick Capture
- Global shortcut opens command-line style input
- Spotlight-style overlay window
- Natural language friction logging
- Enter to submit and close instantly
- Optional dot commands

Suggested commands:
- `.w` = waiting on something
- `.b` = others are waiting on me
- `.r` = repeated task
- `.c` = coordination overhead
- `.g` = guilt pile item

Examples:
- `.w waiting on API approval`
- `.c chased PM twice for decision`
- `.r pasted ticket IDs into Slack again`

### B. Passive Context Capture
- Poll active focused application at modest intervals
- Record only meaningful changes
- Optionally store window title
- Attach passive context to manual logs
- Derive sessions from focus changes rather than storing every poll forever

Initial MVP recommendation:
- app name: yes
- window title: optional, behind explicit user consent or setting

### C. Pattern Detection
- Automatic tagging of manual notes
- Clustering of semantically similar frustrations
- Basic detection of:
  - repeated tasks
  - coordination loops
  - context-switching patterns
  - waiting periods
- Confidence score for inferred tags/patterns

### D. Review Screen
- Daily and weekly summaries
- Top friction clusters
- Time/frequency estimates
- Examples that contributed to each cluster
- User triage:
  - Ignore
  - Watch
  - Worth fixing

### E. Automation Brief Generation
Generate a structured brief from a friction cluster.

Required brief sections:
- Problem
- Trigger
- Current workflow
- Apps involved
- Frequency
- Estimated time cost
- Emotional cost
- Dependencies / blockers
- Example instances
- Desired outcome
- Constraints
- Candidate automation approaches
- Agent-ready prompt/spec

### F. Export / Handoff
- Copy as plain text
- Copy as agent spec
- Export as Markdown
- Optional “Send to agent” integration later

### G. Privacy Controls
- Pause tracking
- Exclude apps
- Choose app-only or app-plus-title mode
- Data retention controls
- View and delete collected data

### H. Closed Loop
- Mark friction cluster as:
  - Resolved
  - Reduced
  - Unchanged
- Ask follow-up after a brief has been generated and exported
- Track estimated recovered time after intervention

## Post-MVP Features
- Real-time nudges and toasts
- Recipe capture mode
- Friction heatmap
- Cross-device sync
- Native integrations with specific AI agents
- Suggested automations using external APIs
- Team/shared friction analysis
- “Record the steps next time” workflow capture

## 12. Key UX Requirements

### Quick Capture UX
- Open in under 150ms target
- Single input focus on open
- Close immediately on Enter
- Async processing after dismissal
- Minimal visual chrome
- No mandatory fields

### Review UX
- Separate from capture mode
- Calm and reflective, not interruptive
- Weekly digest should prioritize top 3 opportunities
- Each cluster should be auditable with examples
- Users should be able to correct tags easily

### Tone
The app should never shame the user.

Avoid:
- “You are distracted”
- “You wasted 3 hours”
- “You failed to focus”

Prefer:
- “This appears to recur often”
- “This may be a good candidate for automation”
- “This pattern looks coordination-heavy”
- “Worth fixing?”

## 13. Nudge Strategy

Real-time nudges should not be central to the MVP.

### Recommendation
Start with:
- end-of-day summaries
- weekly reviews
- optional reflection prompts

Only introduce real-time nudges for high-confidence patterns.

### Example future nudge
“If you switched between VS Code and Stack Overflow 15 times in 10 minutes:
‘High context-switching detected. Was this research, or is this a workflow worth documenting?’”

### Nudge rules
- rare
- dismissible
- teachable
- confidence-based
- never spammy

## 14. Success Metrics

### Primary product metrics
- Weekly active users
- Average manual logs per user per week
- Percentage of users who review clusters weekly
- Percentage of users generating at least one automation brief
- Percentage of generated briefs exported/copied
- Percentage of users marking a cluster as worth fixing
- Percentage of users reporting resolved or reduced friction

### Quality metrics
- Average time to submit a log
- Nudge dismissal rate
- Precision of inferred categories based on user correction
- Share of generated briefs rated useful
- False-positive rate for passive pattern detection

### Trust metrics
- Percentage of users configuring exclusions/privacy settings
- Opt-in rate for title capture
- Retention after privacy onboarding
- Support requests or complaints about creepiness/privacy

## 15. Functional Requirements

### FR1: Manual friction logging
The system must allow users to log freeform friction notes through a global shortcut.

Acceptance criteria:
- Shortcut opens quick-capture input
- User can submit with Enter
- Input disappears immediately
- Log is stored with timestamp

### FR2: Passive desktop context
The system must capture active app context in the background.

Acceptance criteria:
- Focused app is sampled every 2–5 seconds
- Only changes are persisted
- Context is associated with manual logs
- User can disable title capture
- User can exclude apps from tracking

### FR3: Waiting and blocking markers
The system must support low-friction state markers.

Acceptance criteria:
- `.w` starts or records a waiting state
- `.b` records that others are waiting on the user
- Durations may be inferred when state toggles are used
- Waiting periods are visible in review mode

### FR4: Categorization
The system must infer friction categories from notes and patterns.

Acceptance criteria:
- Notes are tagged automatically
- User can edit tags
- System stores confidence score
- Categories include repetitive work, coordination tax, context-switching cost, bottleneck, guilt pile

### FR5: Clustering
The system must cluster similar friction logs into patterns.

Acceptance criteria:
- Similar notes can be grouped
- Cluster summaries are generated
- App context can contribute to clustering
- Users can inspect cluster examples

### FR6: Review summaries
The system must provide daily and weekly review surfaces.

Acceptance criteria:
- Top clusters shown by count and estimated cost
- Users can mark clusters as ignore, watch, or worth fixing
- Weekly review highlights top 3 automation candidates

### FR7: Automation brief generation
The system must generate a structured brief from selected clusters.

Acceptance criteria:
- Brief includes required sections
- Brief references examples
- Brief can be copied/exported

### FR8: Privacy controls
The system must provide clear controls over captured data.

Acceptance criteria:
- Pause toggle exists
- Excluded apps list exists
- Title capture can be disabled
- Data export and delete are supported
- Privacy settings are shown during onboarding

### FR9: Outcome tracking
The system must allow the user to close the loop on generated briefs.

Acceptance criteria:
- User can mark a cluster/brief as resolved, reduced, or unchanged
- Follow-up prompt appears after export or after a period of time
- Outcome data is stored for later reporting

## 16. Non-Functional Requirements

### Performance
- Quick-capture window should appear near-instantly
- Manual submission should not block on analysis
- Background polling must not noticeably impact CPU
- Database queries for review summaries should feel immediate

### Reliability
- No loss of submitted notes
- Background focus tracking should recover gracefully after sleep/resume
- App should tolerate OS permission changes

### Privacy and Security
- Local-first by default
- No screenshots
- No keystroke logging
- No content scraping
- Sensitive apps can be excluded
- User should understand exactly what is collected

### Explainability
- Briefs and clusters should show source examples
- Inferred tags should be editable
- Passive inferences should not feel opaque

## 17. Privacy and Trust Requirements

This is a core product area, not an afterthought.

### Explicit privacy commitments
- No screenshots
- No keystroke logging
- No clipboard scraping unless explicitly introduced later
- No content scraping from apps or web pages
- App only captures:
  - timestamps
  - active app name
  - optional window title
  - manual notes
  - derived metadata

### Required controls
- App exclusion list
- App-only mode
- App + title mode
- Pause/resume tracking
- Private session mode
- Retention settings
- Full export
- Full delete

### Sensitive defaults
Recommended:
- default to app-name collection only
- title capture requires explicit consent
- obvious exclusion suggestions for password managers, banking apps, personal messaging apps

## 18. Data Model

Suggested logical entities:

- `focus_events`
  - timestamp
  - app_name
  - window_title
  - normalized_title
  - redaction_status
- `manual_logs`
  - timestamp
  - raw_text
  - parsed_command
  - inferred_tags
  - confidence
  - related_app_context
- `waiting_periods`
  - start_time
  - end_time
  - note
  - direction
- `derived_sessions`
  - app sequence
  - duration
  - switch count
- `friction_clusters`
  - cluster_id
  - title
  - summary
  - category
  - confidence
  - status
- `automation_briefs`
  - brief_id
  - source_cluster_id
  - generated_text
  - exported_at
  - resolution_status
- `privacy_rules`
  - app_name
  - mode
  - exclusion_status
- `user_feedback`
  - corrected_tags
  - ignored_nudges
  - usefulness_ratings

## 19. Technical Approach

### Stack
- Desktop shell: Tauri v2
- Backend: Rust
- Local database: SQLite via `tauri-plugin-sql`
- Global shortcuts: Tauri capability/plugin
- Active window detection: Rust crates such as `active-win` or `x-win`-style alternatives depending on platform support
- Background analysis: async Rust tasks or dedicated worker thread(s)

### Architecture principles
Separate the system into three paths:

1. Capture path
- must be instant
- write note and context quickly
- no blocking analysis on submit

2. Analysis path
- background clustering
- tagging
- session derivation
- digest generation

3. Output path
- review UI
- brief generation
- export

### Focus tracking approach
Do:
- sample every 2–5 seconds
- record only changes
- debounce noisy title changes
- normalize titles
- derive sessions later

Do not:
- store redundant identical events forever
- aggressively aggregate too early

### Storage strategy
- keep raw events for a limited retention period
- store derived summaries for long-term use
- preserve enough sequence information for clustering and pattern analysis

Suggested retention:
- raw focus events: 14–30 days
- derived sessions/clusters/briefs: long-term unless deleted

## 20. Platform Considerations

### macOS
- accessibility permissions may be required to inspect other app/window metadata
- onboarding must clearly explain why permissions are needed
- product should degrade gracefully if permissions are denied

### Windows
- ensure focused window title/process detection works reliably
- test global shortcut conflicts

### Linux
- active window access can vary by compositor/window system
- support expectations may need to be scoped by environment

## 21. Risks and Mitigations

### Risk 1: Product feels creepy
Mitigation:
- local-first by default
- no screenshots/keystrokes
- visible pause controls
- app exclusions
- explicit privacy onboarding

### Risk 2: False positives reduce trust
Mitigation:
- use confidence scores
- emphasize review summaries over live nudges
- allow correction and dismissal
- show source examples behind pattern claims

### Risk 3: Logging becomes too much work
Mitigation:
- one-line input only
- no required forms
- auto-tagging and auto-context
- instant dismissal

### Risk 4: Product drifts into generic productivity tracking
Mitigation:
- center outputs around automation opportunities
- prioritize “worth fixing” over “hours spent”
- avoid scorecards and judgmental analytics

### Risk 5: Users get lots of insights but no action
Mitigation:
- make automation brief the primary artifact
- include copy/export workflows
- follow up on whether friction was reduced

## 22. MVP Milestones

### Milestone 1: Core capture
- global shortcut
- quick-capture input
- local note storage
- app focus tracking
- privacy controls basic version

### Milestone 2: Review and clustering
- note tagging
- friction clustering
- weekly review page
- top friction categories

### Milestone 3: Brief generation
- structured automation brief
- copy/export
- worth-fixing triage

### Milestone 4: Outcome loop
- resolved/reduced/unchanged states
- follow-up prompt
- recovered-time estimate

## 23. Open Questions

- Should window title capture be disabled by default in v1?
- Should weekly review be the default cadence, with daily summary optional?
- Should brief generation run locally, via local model, or via external API?
- How much of the brief should be deterministic vs LLM-generated?
- Should “recipe capture” be included in MVP if it materially improves brief quality?
- What is the minimum useful number of logs before generating the first brief?
- What confidence threshold should be required before showing a real-time nudge?
- Should there be a browser extension later for richer web context, or should desktop-only remain the product boundary?

## 24. Example Brief Output Format

### Automation Brief
Problem:
You repeatedly copy Jira issue IDs and status context into Slack updates.

Trigger:
This occurs when preparing project updates and responding to status requests.

Current workflow:
- Open Jira
- Find relevant issue
- Copy issue ID
- Switch to Slack
- Paste issue ID
- Add status/commentary
- Repeat for multiple tickets

Apps involved:
- Jira
- Slack
- Browser

Frequency:
Estimated 4 times per week

Estimated time cost:
Approx. 45–60 minutes per week

Emotional cost:
Low complexity, high annoyance, repetitive

Dependencies / blockers:
Requires access to Jira issue metadata and target Slack channel conventions

Example instances:
- “Tired of copy-pasting Jira IDs into Slack.”
- “Rewriting the same update format again.”
- App context repeatedly alternates between Jira and Slack

Desired outcome:
Generate Slack-ready updates from selected Jira tickets automatically

Constraints:
- Must preserve formatting
- Should allow manual review before posting
- Should work with current Jira project structure

Candidate automation approaches:
- Script using Jira API + Slack API
- Slash command that pulls selected Jira issue summaries
- AI agent workflow that drafts update text from selected issue list

Agent-ready spec:
“I am an engineer who repeatedly prepares Slack updates from Jira tickets. Build a script or workflow that takes a list of Jira issue IDs, fetches summary/status/assignee, formats them into a Slack-ready update, and lets me review before posting. Preserve project naming conventions and allow easy reuse.”

## 25. Suggested Positioning

Primary headline options:
- Find what to automate before you automate it.
- A friction journal that turns annoying work into agent-ready briefs.
- Track the drag in your day. Turn it into automation.
- See where your work gets stuck — and generate a spec to fix it.

Supporting line:
Log moments of friction, spot patterns, and generate structured briefs for AI agents, scripts, and workflows.

## 26. Summary

Friction Log is a desktop companion for identifying recurring work friction and turning it into action.

Its differentiators are:
- friction-first capture
- psychologically resonant categories
- privacy-first passive context
- structured automation briefs as the core output
- a closed loop that measures whether friction was actually reduced

The MVP should stay tightly focused on:
- lightweight capture
- trustworthy passive context
- useful weekly review
- high-quality automation brief generation

The product should avoid becoming a generic productivity tracker. Its job is to identify recoverable time and convert it into something the user can act on.

If you want, I can next turn this into one of these:
1. a lean one-page PRD for a startup team
2. a detailed engineering spec with architecture and schema
3. a roadmap split into MVP / v1 / v2
4. wireframe-level screen specs for the Tauri app
5. a set of example user journeys and jobs-to-be-done
