
# End-to-End Journey Map

This is the full lifecycle Friction Log should support.

### Stage 1: Notice
The user experiences friction:
- repeated work
- waiting
- switching
- chasing
- avoidance

### Stage 2: Capture
The user logs it quickly:
- freeform note
- optional command shortcut
- passive context attached automatically

### Stage 3: Aggregate
The system groups related moments:
- tags
- clusters
- sessions
- waiting periods
- app patterns

### Stage 4: Review
The user sees:
- top recurring friction
- estimated cost
- categories
- examples
- confidence

### Stage 5: Prioritize
The user decides:
- Ignore
- Watch
- Worth fixing

### Stage 6: Generate
The app creates:
- automation brief
- agent spec
- workflow description

### Stage 7: Act
The user:
- hands off to AI agent
- builds a script
- creates a process change
- delegates the work

### Stage 8: Evaluate
The app asks:
- resolved?
- reduced?
- unchanged?

### Stage 9: Learn
The app improves future suggestions based on:
- user corrections
- ignored patterns
- successful outcomes

## Anti-Journeys: What Should Not Happen

These are useful for product guardrails.

### Anti-journey 1: The app feels like surveillance
User installs the app and immediately worries it is spying on them.

Avoid by:
- local-first defaults
- explicit privacy onboarding
- app-only mode
- visible pause/exclusions

### Anti-journey 2: Logging friction feels like extra friction
User opens capture and sees too many fields or choices.

Avoid by:
- one-line input
- auto-tagging
- no required taxonomy decisions

### Anti-journey 3: The app nags constantly
User receives frequent context-switch nudges that are not helpful.

Avoid by:
- no real-time nudges in early MVP
- confidence thresholds
- summary-first design

### Anti-journey 4: The output is too generic
User gets a brief that sounds clever but cannot be used.

Avoid by:
- source examples
- app context
- structured fields
- editable output

### Anti-journey 5: The app creates guilt instead of clarity
User feels judged for being busy or distracted.

Avoid by:
- compassionate tone
- systems framing
- “worth fixing” instead of “you wasted time”

## 7. Opportunity Areas Derived from the Journeys

These journeys suggest several product opportunities:

### Opportunity 1: Better command language
A small command grammar could make logging even faster.

Examples:
- `.w waiting on legal sign-off`
- `.b team blocked on my review`
- `.r manual release notes again`
- `.g still avoiding monthly cleanup`

### Opportunity 2: Review should emphasize “Top 3”
Users likely need prioritization more than analytics.

A strong review page should answer:
- What repeated most?
- What cost the most?
- What is easiest to fix?
- What should I ignore?

### Opportunity 3: Evidence-first clustering
Trust will increase if every cluster can show:
- notes
- app context
- waiting periods
- frequency
- confidence

### Opportunity 4: Outcome tracking as value proof
The best proof that Friction Log works is:
- “I fixed 3 things this month”
- “Estimated 2.5 hours/week recovered”

## 8. Compact JTBD Summary for Positioning

If you want a short version for internal docs or a pitch deck, use this:

Friction Log helps knowledge workers capture moments of recurring work friction, detect patterns across those moments, and turn them into structured automation briefs that can be handed to AI agents or used to redesign workflows.

Users hire Friction Log to:
- notice what keeps draining time
- decide what is worth fixing
- produce a concrete artifact they can act on
- measure whether the fix worked
