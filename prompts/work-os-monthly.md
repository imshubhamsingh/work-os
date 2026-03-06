---
description: Generate a monthly work summary from weekly summaries with career-level signal, systemic patterns, and promotion-readiness audit
model: opus
---

# Generate Monthly Summary

You are generating a **high-signal monthly work summary in Markdown** from weekly summaries.

This is a **strategy and pattern synthesis task** — not a weekly aggregation, not a task list.

The monthly operates at a higher resolution than the weekly:
- Weekly asks: "Was this a good week?"
- Monthly asks: "What kind of engineer am I becoming? What systemic issues need structural fixes?"

Optimize for career signal, systemic insight, and honest self-assessment.

---

## 🧭 Executive Summary (VP/Director-Readable) — MANDATORY

At the very top, generate a **lead-level narrative summary** (6–8 bullets maximum).

This summary should allow a manager to evaluate the **month** without reading anything else.

Cover:
- Overall month sentiment *(high-intensity / steady / recovering / stressful)*
- Delivery health *(on-track / stretched / at-risk, name the main deliverable)*
- Focus split *(execution vs reviews/mentorship vs coordination)*
- The single most impactful thing shipped this month
- Systemic risk or recurring pattern worth addressing
- Oscillation health for the month *(Oscillating / Mixed / Linear — derived from weekly patterns)*
- Career trajectory signal *(growing scope / holding steady / narrowing)*
- AI leverage *(is AI amplifying output meaningfully?)*

Rules:
- Bullet points only, no checkboxes
- No emojis, no task-level detail
- Objective, honest, calm tone
- Reads like a **staff / principal engineer monthly narrative**
- Do NOT include specific PR numbers or dates in this section
- Source links are not needed here — details go in the body

---

## Data Sources (STRICT)

### Primary Source — Weekly Summaries

Read all weekly summary files from:

```
$WORK_OS_BASE_DIR/archive/weekly/
```

**Use Bash ls to list files** (do NOT use Glob — it silently fails on `~` paths):
```bash
ls $WORK_OS_BASE_DIR/archive/weekly/
```

Include a week if **any of its days fall within the target month**. Do not exclude partial weeks.

Example for February 2026: include 2026-01-25_2026-01-31.md if it has Feb 1 days... but since it ends Jan 31, exclude it. Include 2026-02-02_2026-02-07.md through 2026-02-22_2026-02-28.md.

**How to determine target month:**
- Default: the most recently completed calendar month relative to today
- If today is the last day of the month (or within the first 2 days of a new month), generate for the month just ended
- Otherwise generate for the prior calendar month

### Secondary Source — Prior Monthly (for comparison)

Check if a prior monthly exists:
```
$WORK_OS_BASE_DIR/archive/monthly/{PRIOR_MONTH}.md
```
e.g. for February summary, check `archive/monthly/2026-01.md`.

If it exists, read the `## 📊 Month at a Glance` and `## 🤖 AI Adoption Trajectory` sections for comparison data. Do NOT read the full file — just those sections.

If it doesn't exist, skip silently and omit the month-over-month comparison column.

### Supplemental Source — Selective Raw / Archive Access

When weekly summaries lack sufficient signal for specific sections (delegation evidence, AI leverage types, stress attribution), you may selectively read additional files. Use judgment — do not read files wholesale.

**When to reach into raw/:**
- **AI Leverage types** (item 8): scan raw sync files for ccusage PR breakdowns if the weekly AI % doesn't break down by usage type. Look at `raw/{DATE}/sync-*.md` — the `📊 [GITHUB] AI Usage:` blocks describe individual PR commit patterns.
- **Delegation evidence**: scan raw sync files for DMs or Slack threads where you handed off work, asked someone to pick something up, or explicitly chose NOT to own something.
- **Stress attribution**: if it's unclear whether after-hours work was externally deadlined vs. self-inflicted, check raw sync Slack activity timestamps vs. meeting MOMs in the same day.

**When to reach into archive/ (individual daily files):**
- Only for a specific week when the weekly summary's signal is ambiguous
- Read only the specific daily file for the relevant date — not all daily files for the month

**How to discover available files:**
```bash
ls $WORK_OS_BASE_DIR/raw/{MONTH_RANGE}/
ls $WORK_OS_BASE_DIR/archive/daily/
ls $WORK_OS_BASE_DIR/archive/weekly/
```

Read selectively — the Bash `ls` first to see what exists, then Read only the specific file you need. Never read all raw files for the month — it's too expensive and the weekly summaries already synthesize them.

---

## Obsidian Source Linking

Link to **weekly summary files** as sources, not daily briefs.

**Preferred format for weekly source links:**
```markdown
[[weekly/2026-02-22_2026-02-28|W4 source]]
[[weekly/2026-02-15_2026-02-21|W3 source]]
```

**In table cells, always escape the pipe:**
```markdown
[[weekly/2026-02-22_2026-02-28\|W4]]   ✅ correct
[[weekly/2026-02-22_2026-02-28|W4]]    ❌ breaks the table
```

Use week labels (W1, W2, W3, W4, W5) for brevity. W1 = first week of the month with the most days in it.

---

## Time Allocation Derivation

For each weekly summary, extract the following signals to build the time allocation estimate.

### Step 1: Parse the Focus bullet

In each weekly's `## 🧭 Lead Summary`, find the `**Focus:**` bullet.
It reads like: *"~50% hands-on execution (Feature X fixes), ~35% reviews, ~15% coordination"*

Extract the percentages for three categories: **Execution**, **Reviews**, **Coordination**.
If a week's focus bullet is missing or unclear, use the surrounding snapshot data (PRs authored vs reviews vs meetings mentioned) to estimate.

### Step 2: Detect firefighting embedded inside "Coordination"

Firefighting is **not** broken out in the Focus bullet — it hides inside "coordination." Extract it separately by counting firefighting signals from each weekly's body:

**Firefighting signals (count per week):**
- Any item described as "drop everything", "urgent unplanned", or unexpected P0 that appeared same-day: +8%
- `@hub-oncall` or `#your-bug-alert-channel` appearances requiring response: +3% per incident
- Reactive user-support responses from `#your-support-channel` or `#temp-ilf-bugs` that were unplanned: +2% per cluster
- Day's top-3 outcomes replaced mid-day by urgent incoming: +5%
- Cap total firefighting per week at 25%

**Re-normalize:** Subtract the estimated firefighting % from Coordination % for that week.
If firefighting exceeds Coordination, also subtract from Execution.

### Step 3: Aggregate across weeks

Average the four category values across all weeks in the month (equal weight per week).
Round each final % to the nearest 5% — these are directional signals, not audit-grade data.

### Step 4: Calculate gaps

**Target allocation (your personal targets):**
| Category | Target |
|---|---|
| Deep execution | 50% |
| Reviews | 15% |
| Coordination | 20% |
| Firefighting | 5% |

Gap = Estimated % - Target %. Negative gap = underallocated vs target. Positive gap = overallocated.

**Gap severity thresholds:**
- `> +10%` or `< -10%`: Structural drag — must appear in Systemic Patterns
- `+5% to +10%`: Worth flagging — include an observation
- Within ±5%: Within normal variance — no action needed

### Step 5: Build the per-week breakdown table

For each week, record the four estimated %s. This shows where allocation shifted week to week and what caused it (e.g., a release week spikes firefighting; a review blitz week spikes Reviews and compresses Execution).

---

## AI Stats Aggregation

For each weekly summary, extract the `## 🤖 Week's AI Usage Statistics` section.

Parse:
- Weekly Total LOC, AI LOC, Human LOC, AI %

Aggregate to monthly:
- Sum all Total LOC, AI LOC, Human LOC
- Monthly AI % = (Total AI / Total Monthly LOC) * 100

Also extract the per-day AI comparison tables from each week for the **AI Trajectory** section.

If no AI stats found in any weekly, omit the AI sections entirely.

### AI Leverage Quality Derivation

Beyond the raw AI %, classify *how* AI was used each week. Scan weekly PR descriptions, commit messages in sync files, and review comments for patterns.

**Leverage categories:**
- **Boilerplate acceleration** — AI wrote repetitive scaffolding (API clients, types, CRUD hooks). Signal: large LOC, few commits, routine patterns.
- **Refactor assistant** — AI helped migrate/restructure existing code. Signal: high churn (lines deleted ≈ lines added), module migration PRs.
- **Architecture exploration** — AI helped think through design options before implementation. Signal: plan files, design doc PRs, back-and-forth in commit messages.
- **Test generation** — AI wrote test cases. Signal: test file changes, `.test.ts` files in PRs.
- **Debugging** — AI helped diagnose a bug. Signal: fix PRs with small LOC but high cognitive value.
- **Spec drafting** — AI wrote requirements, MOM summaries, or technical docs. Signal: markdown file changes, non-code PRs.

For each week, estimate the dominant leverage type (the one that drove the most AI LOC or highest cognitive value). Record as a distribution across the month.

**Key question:** Was AI used to tackle problems you couldn't have handled alone, or just to move faster on things you already knew how to do? The former = leverage. The latter = acceleration. Both are good — but the ratio matters for career growth.

### Delegation Score Derivation

Scan weekly summaries (and selectively raw DMs/Slack) for delegation signals:

**Delegation happened when:**
- A task was explicitly handed to a team member with context
- A review was assigned to a team member who could handle it without you
- A recurring responsibility was transferred to someone else
- A decision was pushed down to the appropriate owner instead of escalated up

**Non-delegation (missed opportunity) signals:**
- you reviewed a PR that a junior team member could have reviewed
- you fixed a bug that could have been a growth opportunity for someone else
- A meeting attendance that wasn't necessary for you's decision-making
- End-of-day reflections mentioning "should have delegated" or "why did I do this?"

Count both delegated and missed-delegation items per month. Track:
- `Delegation ratio` = delegated / (delegated + could-have-delegated)
- Target: ≥ 30% (as lead, roughly 1 in 3 doable items should route to the team)

---

### Role Load Derivation — EM × Staff × IC

You are currently operating across three roles simultaneously. Derive the monthly split from weekly summaries.

**What counts as each role:**

| Role | Signals to scan for |
|------|-------------------|
| **Engineering Manager (EM)** | 1:1s held, career conversations, interviews conducted, candidate feedback, onboarding new hires, team health discussions, performance inputs, hiring pipeline meetings, stakeholder representation on behalf of team |
| **Staff Engineer** | Architecture decisions, design docs authored/reviewed, FE/BE standards sessions, cross-team unblocking, tooling PRs that scale (bot rules, CI, module structure), data model decisions, technical strategy alignment, code reviews that are quality/standards-focused |
| **IC** | PRs authored, bug fixes (especially same-day), hands-on feature implementation, late-night coding sessions, debugging, routine PR approvals |

> Code reviews are ambiguous — classify as **Staff** if they involved architectural feedback or set a standard; classify as **IC** if they were routine approvals.

**Derivation steps:**

1. For each week, scan the weekly summary for EM, Staff, and IC signals. Count distinct activities per role.
2. Estimate time weight per activity type:
   - Each interview or 1:1: ~1.5h → EM
   - Each architecture/ERD session: ~2h → Staff
   - Each PR authored (merged): ~3h average → IC
   - Each tooling/standards initiative: ~2h → Staff
   - Each bug fix: ~1h → IC
3. Compute % share per role per week. Normalize to 100%.
4. Average across 4 weeks for monthly split. Round to nearest 5%.

**Target allocation:**

| Role | Target | Rationale |
|------|--------|-----------|
| EM | 20% | Enough to be a real manager — 1:1s, hiring, team health — without becoming a pure people manager |
| Staff | 40% | Highest-leverage zone — architecture, standards, cross-team impact |
| IC | 40% | Staying technically credible without becoming the bottleneck coder |

**Gap severity thresholds:**
- IC > 55%: structural — Staff and EM work is being crowded out by hands-on execution
- EM > 35%: structural — becoming manager-first, losing technical leverage
- Staff < 25%: structural — not enough leverage work; individual heroics dominating

**Role conflict events** — scan EOD reflections for moments where two roles competed for the same time:
- Interview day that overlapped with a release crunch → EM vs IC
- 1:1 that displaced architectural alignment → EM vs Staff
- Late-night coding that displaced team communication → IC vs EM

---

## Output Location (MANDATORY)

Write the monthly summary to:

```
$WORK_OS_BASE_DIR/archive/monthly/{YYYY-MM}.md
```

Rules:
- Create the `monthly/` directory if it does not exist
- Format: `YYYY-MM.md` (e.g. `2026-02.md`)
- **Never overwrite an existing file** — if it exists, exit with a message

---

## Step 0 — Narrative Check (Pre-Write Gate)

Before writing a single section, answer these four questions internally. They set the framing for the entire document:

1. **If my manager forwarded this to the VP, would I look strategic?** If the answer is "probably not", the month had too much tactical noise — make sure the Executive Summary focuses on systems and scope, not PRs.
2. **Does this month read like an IC summary or an org-level engineer?** Count: how many sections talk about code I wrote vs. systems I improved vs. people I unblocked? If it's >60% code, the framing needs adjustment.
3. **Did I raise the ceiling or survive the month?** Honest answer. If "survived" — say so. Executives respect honesty. Self-congratulating summaries under pressure signal poor self-awareness.
4. **What would a principal engineer have done differently?** One concrete answer. If you can't answer this, you're too close to the work. Take 60 seconds to zoom out.

These answers should shape the tone of the Executive Summary and the Gaps section of Promotion Readiness.
Do NOT print these answers in the output — they're internal framing only.

---

## Output Structure (EXACT)

```markdown
# 📅 Monthly Summary — {MONTH_NAME} {YEAR}

## 🧭 Executive Summary
- **Sentiment:** [Month-level energy, one line]
- **Delivery:** [Primary deliverable outcome]
- **Focus:** [Execution vs reviews vs coordination — monthly split]
- **Key win:** [Single most impactful outcome of the month]
- **Systemic risk:** [The pattern that kept recurring — what caused it?]
- **Oscillation:** [Monthly health: Oscillating 🟢 / Mixed 🟡 / Linear 🔴]
- **Career signal:** [Growing / Holding / Narrowing scope — brief reason]
- **AI leverage:** [Is AI meaningfully amplifying output? One line]

> This section must stand alone. A director should understand the month without reading further.

---

## 📊 Month at a Glance

| Metric | {MONTH} | {PRIOR_MONTH} (if available) | Trend |
|--------|---------|------------------------------|-------|
| Weeks covered | X | X | |
| PRs authored | X | X | ↑/↓/→ |
| PRs merged | X | X | ↑/↓/→ |
| Reviews completed | X | X | ↑/↓/→ |
| After-hours nights | X | X | ↑/↓/→ |
| Total LOC | X,XXX | X,XXX | ↑/↓/→ |
| AI % | XX% | XX% | ↑/↓/→ |
| Carryover peak | X | X | ↑/↓/→ |
| Oscillation | 🟢/🟡/🔴 | 🟢/🟡/🔴 | |

> Omit the prior-month column entirely if no prior monthly file exists.
> Trend: ↑ improving, ↓ declining, → stable.

---

## ⏱️ Time Allocation Reality Check

> Productivity issues are rarely effort problems — they're allocation problems.
> This table shows where time actually went vs. where it should go.
> Numbers are directional estimates derived from weekly Focus bullets and firefighting signals (see derivation instructions above).

### Monthly Average

| Category | Estimated % | Target % | Gap | Verdict |
|----------|------------|----------|-----|---------|
| Deep execution | XX% | 50% | ±X% | On target / Over / Under |
| Reviews | XX% | 15% | ±X% | On target / Over / Under |
| Coordination | XX% | 20% | ±X% | On target / Over / Under |
| Firefighting | XX% | 5% | ±X% | On target / Over / Under |

**Total:** 100%

> Gap verdict: **On target** = within ±5% · **Over** = consuming more than target · **Under** = less than ideal
> Firefighting gap > +10%: structural. Flag in Systemic Patterns.
> Execution gap < -10%: high-leverage fix needed. Will also show in oscillation data.

### Per-Week Breakdown

| Week | Execution | Reviews | Coordination | Firefighting | Heaviest drain |
|------|-----------|---------|--------------|--------------|----------------|
| W1 | XX% | XX% | XX% | XX% | [what drove the imbalance] |
| W2 | XX% | XX% | XX% | XX% | [what drove the imbalance] |
| W3 | XX% | XX% | XX% | XX% | [what drove the imbalance] |
| W4 | XX% | XX% | XX% | XX% | [what drove the imbalance] |
| **Avg** | **XX%** | **XX%** | **XX%** | **XX%** | |

📊 **Allocation narrative:** [2–3 sentences. Which category was chronically over-budget? Which week was the worst? What specific event or pattern caused the biggest drift from target? Did any week get allocation right?]

> **How to read this:**
> - "Heaviest drain" = the single thing that consumed disproportionate time that week (a release, a review blitz, oncall incidents, a stalled feature)
> - Firefighting is extracted separately from Coordination — it hides inside "coordination" in weekly Focus bullets but has a different fix than planned meetings
> - These estimates are ±10% precise. Use them for directional insight, not accounting.

---

## 🚀 Features Shipped — Full Lifecycle

Group by feature/product area. Show the arc from kick-off to production.

### [Feature Name]
- **Arc:** [Concept / ERD / Build / Review / Testing / Shipped] — X weeks
- **Week shipped:** W{N}
- **PRs:** [list with links, one per line — format: `[repo-name #N](url)`]
- **Impact:** [Who uses it? What problem does it solve? One sentence]
- **Source:** [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]]

> **What qualifies as a shipped feature:**
> - Deployed to production with measurable user impact
> - A complete technical initiative (tooling, process, architecture)
>
> **What does NOT qualify:**
> - Bug fixes (list separately below)
> - Carryover items not yet shipped
> - Refactors without user-facing output

### Bug Fixes & Maintenance (brief list)
- [Bug description] — [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]]

---

## 💻 Code Output Summary

**Monthly:** {TOTAL} LOC authored | {AI} AI ({AI_PCT}%) | {HUMAN} Human ({HUMAN_PCT}%)

`[Progress Bar 10 blocks]` {AI_PCT}% AI Generated

**PRs authored:** X total, Y merged

### Weekly Breakdown

| Week | PRs Authored | Merged | Total LOC | AI % |
|------|--------------|--------|-----------|------|
| W1 (MM-DD → MM-DD) | X | X | X,XXX | XX% |
| W2 (MM-DD → MM-DD) | X | X | X,XXX | XX% |
| W3 (MM-DD → MM-DD) | X | X | X,XXX | XX% |
| W4 (MM-DD → MM-DD) | X | X | X,XXX | XX% |
| **Total** | **X** | **X** | **X,XXX** | **XX%** |

---

## 👀 Review Throughput

**Monthly reviews:** X completed

- **Average per week:** X
- **Repos covered:** [list repos]
- **Throughput pattern:** [spike-heavy / steady / light — and when spikes occurred]
- **Structural insight:** [Was review load sustainable? Any bottlenecks repeated?]

| Week | Reviews Done | Longest Pending | Notes |
|------|-------------|-----------------|-------|
| W1 | X | [repo-name #N (~Xmo)](url) | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]] |
| W2 | X | [repo-name #N (~Xmo)](url) | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]] |
| W3 | X | [repo-name #N (~Xd)](url) | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]] |
| W4 | X | [repo-name #N (~Xd)](url) | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]] |

---

## 🤖 AI Adoption Trajectory

**Monthly average:** {AI_PCT}% AI-generated

`[Progress Bar 10 blocks]` {AI_PCT}%

### Weekly AI % Trend

| Week | AI LOC | Human LOC | AI % | vs Prior Week |
|------|--------|-----------|------|---------------|
| W1 | X,XXX | X,XXX | XX% | — |
| W2 | X,XXX | X,XXX | XX% | ↑/↓/→ |
| W3 | X,XXX | X,XXX | XX% | ↑/↓/→ |
| W4 | X,XXX | X,XXX | XX% | ↑/↓/→ |

📊 **Monthly pattern:** [Narrative — was AI adoption rising, falling, or spiking on specific work types? What drove the highest AI weeks? When did human contribution dominate and why?]

### 🔬 AI Leverage Quality

> AI % gamifies into high-volume boilerplate if not checked. This table tracks *how* AI was used — the signal that matters for career growth.

| Leverage Type | Weeks Active | Dominant Week | Quality Signal |
|---|---|---|---|
| Boilerplate acceleration | X/4 | W{N} | Low — faster, not bigger |
| Refactor assistant | X/4 | W{N} | Medium — improves existing |
| Architecture exploration | X/4 | W{N} | High — tackles novel problems |
| Test generation | X/4 | W{N} | Medium — improves reliability |
| Debugging | X/4 | W{N} | High — cognitive leverage |
| Spec / doc drafting | X/4 | W{N} | High — multiplies team clarity |

**Leverage verdict:** [Did AI expand the ceiling of what you could tackle, or primarily accelerate what you already knew how to do? One honest sentence. If mostly boilerplate — name it. If architecture exploration — that's the signal to amplify.]

> **How to populate:**
> - Use weekly AI stats, commit messages, and PR descriptions as signals
> - A week with 90% AI on a module migration = Refactor
> - A week with 90% AI on a new architecture = Architecture exploration
> - If unclear, mark as "Boilerplate" as the conservative estimate

> AI % reflects coding output only. Reviews, coordination, and planning are not measured here but are equally part of the role.

---

## 🔄 Oscillation Summary

**Monthly pattern:** Oscillating 🟢 / Mixed 🟡 / Linear 🔴

*Derived from the weekly oscillation ratings across the month.*

| Week | Rating | After-Hours Nights | Carryover Peak | Signal |
|------|--------|-------------------|----------------|--------|
| W1 | 🟢/🟡/🔴 | X | X | [brief] |
| W2 | 🟢/🟡/🔴 | X | X | [brief] |
| W3 | 🟢/🟡/🔴 | X | X | [brief] |
| W4 | 🟢/🟡/🔴 | X | X | [brief] |

**Total after-hours nights:** X / {WORKDAYS} work days ({PCT}%)

**Verdict:** [2–3 sentences. Was the month sustainable? What drove the worst weeks? Did any recovery happen?]

### Stress Attribution

> Oscillation is descriptive by default. This makes it diagnostic.
> Classify what *caused* the after-hours nights and load peaks.

| Stress Source | Estimated % of Load | Evidence Week(s) |
|---|---|---|
| External deadline / release crunch | XX% | W{N} |
| Self-inflicted overcommitment | XX% | W{N} |
| Unclear scope / requirements churn | XX% | W{N} |
| Review bottleneck (waiting on or for) | XX% | W{N} |
| Reactive firefighting (unplanned P0) | XX% | W{N} |

**Diagnosis:** [Which source was dominant? If "self-inflicted overcommitment" > 30%, that's a senior maturity gap — commit less, execute better. If "external deadline" > 50%, the system needs buffer. One diagnostic sentence per major contributor.]

**Month-level recommendation:**
- [ ] [One structural change to improve next month's oscillation health]

---

## 📈 Promotion Readiness Signals

> This section answers: *"If my perf review was written from this month's data, what story would it tell?"*
> Evaluate against a staff / principal engineer bar — not just "did I ship" but "did I raise the ceiling."

### Work Type Distribution (Staff vs IC)

| Work Type | Count | Career Signal | Source |
|---|---|---|---|
| Feature delivery (owned end-to-end) | X | Senior | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |
| Process / system improvement | X | Staff | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |
| Cross-team unblock / mentorship | X | Staff | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |
| Org-level leverage (tooling, standards, docs) | X | Principal | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |
| Reactive IC (bug fix, support, unplanned) | X | Below Senior | |

**Leverage ratio:** [Staff + Principal items] / [Total items] = XX%
**vs prior month:** XX% → XX% (↑/↓/→)

> **Did the ratio of leverage work increase this month?** If not, you're optimizing for output — not impact.
> Target: ≥ 40% of work items at Staff level or above.
> If Reactive IC > 20%, flag in Drag Log.

### IC Contribution
- [What I owned end-to-end — from design to production]
  [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]]

### Team Leverage
- [PRs I reviewed that unblocked team delivery]
- [Process improvements I introduced or championed]
- [Knowledge-sharing — docs, demos, announcements]
  [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]]

### Cross-Team Influence
- [Impact on teams outside my direct pod]
- [Initiatives that spread beyond the immediate team]
  [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]]

### Gaps / Growth Areas
- [Where I operated below the expected staff bar]
- [What I over-retained that should have been delegated]

> Rules: Promo packets that omit gaps signal low self-awareness. Be specific — "reviewed PRs I should have delegated" is better than "could have delegated more."

---

## 🎭 Role Load Distribution — EM × Staff × IC

> You are currently operating across three roles simultaneously: Engineering Manager, Staff Engineer, and IC.
> This section makes the split visible so you can manage it — not just survive it.
>
> **The core risk:** each role has legitimate urgency. Without explicit tracking, the loudest role wins —
> typically IC (PRs, bugs) or EM (team needs) — while Staff work (architecture, standards, leverage) gets squeezed.

### Role Definitions

| Role | What counts |
|------|------------|
| **Engineering Manager** | 1:1s, career conversations, interviews, onboarding, team health, hiring pipeline, performance inputs, stakeholder representation |
| **Staff Engineer** | Architecture decisions, technical standards, cross-team unblocking, tooling/process that scales, design docs, data model decisions, quality-focused code reviews |
| **IC** | PRs authored, bug fixes, hands-on feature implementation, debugging, routine approvals |

### Monthly Split

| Role | Estimated % | Target % | Gap | Verdict |
|------|------------|----------|-----|---------|
| Engineering Manager | XX% | 20% | ±X% | On target / Over / Under |
| Staff Engineer | XX% | 40% | ±X% | On target / Over / Under |
| IC | XX% | 40% | ±X% | On target / Over / Under |

`[Progress Bar 10 blocks]` XX% Engineering Manager
`[Progress Bar 10 blocks]` XX% Staff Engineer
`[Progress Bar 10 blocks]` XX% IC

**Structural tension:** [One sentence — e.g. "IC crowding out Staff this month — 3 weeks of PR-heavy execution compressed architectural work" or "EM load spiked W2 due to interviews and onboarding, recovered by W4"]

### Per-Week Role Breakdown

| Week | EM % | Staff % | IC % | Heaviest role drain |
|------|------|---------|------|---------------------|
| W1 | XX% | XX% | XX% | [what drove the imbalance] |
| W2 | XX% | XX% | XX% | [what drove the imbalance] |
| W3 | XX% | XX% | XX% | [what drove the imbalance] |
| W4 | XX% | XX% | XX% | [what drove the imbalance] |
| **Avg** | **XX%** | **XX%** | **XX%** | |

### Role Conflict Events

> Moments when two roles directly competed for the same time slot.
> These are the highest-cost context switches — one role always loses.

| Event | Roles in conflict | Winner | Cost |
|-------|------------------|--------|------|
| [e.g. Interview day during release crunch] | EM vs IC | IC | [what was delayed or dropped] |
| [e.g. 1:1 during architecture alignment] | EM vs Staff | EM | [what was delayed or dropped] |

> Omit this table if no clear conflicts observed. Do not manufacture examples.

**Verdict:** [2–3 sentences. Which role dominated? Was that intentional or reactive? What would a more deliberate split have looked like this month?]

> **Gap thresholds:**
> - IC > 55%: structural — Staff and EM work is being crowded out
> - EM > 35%: structural — becoming manager-first, losing technical leverage
> - Staff < 25%: structural — not enough leverage work, individual heroics dominating
> - If any gap > 15% from target: must appear in Systemic Patterns

---

## 🌱 Ceiling Raise Index

> At senior+ levels, productivity ≠ code shipped.
> This metric asks: **how many things I did this month permanently make the team faster next month?**
>
> If this number is 0, you operated tactically. This is the highest-leverage metric for staff growth.

| Ceiling-Raise Action | Type | Permanence | Source |
|---|---|---|---|
| [e.g. Added Storybook rule to VulcanHO] | Tooling | Permanent | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |
| [e.g. Established module structure convention] | Architecture | Permanent | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |
| [e.g. Documented error code contract with BE] | Process | Durable | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |
| [e.g. Onboarded reviewer to PROJECT reviews] | People | Durable | [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] |

**Ceiling Raise Count:** X items

**vs prior month:** X → X (↑/↓/→)

**Verdict:** [Was this a building month or a shipping month? One sentence. "Built X things that compound" vs "Shipped X things that don't repeat."]

> **Types:**
> - **Permanent** — once done, irreversible improvement (linting rule, architecture decision, documented contract)
> - **Durable** — requires occasional maintenance but lasts months (a trained reviewer, an adopted process)
> - **One-time** — valuable but doesn't compound (a single bug fix, a one-off sync)
>
> Only Permanent and Durable count toward the Ceiling Raise Index.

---

## 🎯 Delegation Score

> Track delegation explicitly to avoid regressing into IC-heavy work by default.

| Work Item | Could Delegate? | Delegated? | Why Not (if not) |
|---|---|---|---|
| [e.g. Reviewed minor UI PR] | Yes | No | Habit — didn't think to route |
| [e.g. Fixed small bug reported in #eos-open] | Yes | Yes | Routed to a team member |
| [e.g. Feature X QA fixes] | No | N/A | Owned the feature |

**Delegation ratio:** X delegated / X could-have-delegated = XX%
**Target:** ≥ 30%
**Status:** 🟢 On target / 🟡 Below / 🔴 Regressing (< 20%)

**Pattern:** [One sentence. What types of work did you retain that you shouldn't have? What made you not delegate — habit, trust, speed, or unclear ownership?]

> A delegation ratio < 20% means you're holding scope, not growing it.
> The goal is not to avoid work — it's to route it to the person for whom it's the highest growth opportunity.

---

## 🔁 Systemic Patterns (Recurring ≥ 3 Weeks)

> Patterns that appeared in 3+ weekly summaries are systemic, not situational.
> Weekly nudges haven't fixed them — they need structural interventions.

### [Pattern Name] — [3/4 weeks / all 4 weeks]
**What happened:** [1–2 sentence description of the recurring signal]
**Root cause hypothesis:** [Why does this keep happening?]
**Structural fix:** [Concrete system-level change — not a personal habit]
Sources: [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]] · [[weekly/YYYY-MM-DD_YYYY-MM-DD\|W{N}]]

> **What makes something a systemic pattern:**
> - Appears in ≥ 3 weekly summaries under Misses, Carryovers, or Oscillation
> - Is NOT caused by a single external event (sick day, release crunch, etc.)
> - Has been named as a risk/learning in prior weeks without resolution
>
> **Minimum 1 pattern, maximum 4 patterns.** If fewer than 3 recurrences, do not call it systemic.

---

## 🪨 Drag Log (Friction That Repeated)

> Your current system identifies patterns — but doesn't track whether fixes were attempted.
> Without this, the same friction repeats for 3 months while every weekly says "next week I'll fix it."

| Friction Type | Weeks Seen | Fix Attempted? | Outcome |
|---|---|---|---|
| Context switching / interruption spikes | X/4 | Yes / No / Partial | [what happened] |
| Review overload | X/4 | Yes / No / Partial | [what happened] |
| Vague requirements / scope churn | X/4 | Yes / No / Partial | [what happened] |
| Competing P0s from multiple stakeholders | X/4 | Yes / No / Partial | [what happened] |
| Late-stage feedback after build | X/4 | Yes / No / Partial | [what happened] |
| [Any other friction source seen ≥ 2 weeks] | X/4 | Yes / No / Partial | [what happened] |

**Fix attempt rate:** X / X friction types had a fix attempt = XX%

**Unaddressed drag:** [Which friction types had no fix attempt? One sentence. These are the ones that will be back next month.]

> **Rules:**
> - Only include friction seen in ≥ 2 weeks out of 4
> - "Fix attempted" = a concrete action taken (not just "noted in weekly reflection")
> - If fix was "Partial" — describe what was done and what's still open
> - This section feeds directly into Next Month Commitments — every "No" here should appear as a commitment below

---

## ⏭️ Carry-Forward Momentum

> Work started this month that will pay off next month. Distinct from carryovers (those are debt). This is investment.

**In-flight features / initiatives:**
- [Feature or initiative that's mid-execution, expected to ship next month]
  [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]]

**Structural improvements started:**
- [Process or tooling improvement that's partially adopted — will compound]
  [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]]

**Relationships / knowledge built:**
- [Cross-team connection or domain understanding that will pay off]

> **Carry-forward ≠ carryover.** Carryovers are debt (things that slipped). Carry-forward is compounding investment.

---

## 🧠 Month's Best Learnings

> Pick the 3–5 learnings with the highest re-use value. These should generalize beyond this month.

1. **[Generalizable principle]** — [One sentence: why this matters beyond this month]
   [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]]

2. **[Another principle]** — [Why it generalizes]
   [[weekly/YYYY-MM-DD_YYYY-MM-DD\|source]]

> **Ordering:** highest-leverage first. Cut anything that doesn't generalize beyond this specific month.
> These should read like engineering principles, not lessons from a single incident.

---

## ❌ Kill List — What I Stopped Doing

> Productivity increases more from subtraction than from optimization.
> Senior engineers drown in "helpful" work. This section forces the question: what did I consciously stop?

| What I Stopped | Reason | Impact Observed |
|---|---|---|
| [e.g. Reviewing minor UI-only PRs] | Too low-signal for my time | Routed to a team member — same quality, frees ~30 min/week |
| [e.g. Attending all PROJECT syncs] | Not decision-relevant | Async updates sufficient |
| [e.g. Writing over-detailed PR descriptions] | AI handles boilerplate | No drop in review clarity |

**Kill count:** X items stopped this month

**What I should stop next month but haven't yet:** [One item. The honest one. The thing you're still doing out of habit or guilt.]

> **Rules:**
> - If Kill List is empty, this is a signal — you added more than you removed this month
> - "Stopped" means a deliberate, sustained change — not just skipping something once
> - If something you stopped caused a problem, note it honestly here

---

## 🎯 Next Month: Commitments

> Derived directly from Systemic Patterns. Max 3. Must be structural changes, not habit nudges.

1. **[Structural commitment]** — [One sentence: what changes and why it will break the pattern]
2. **[Another commitment]**
3. **[Third commitment]**

> **Rules:**
> - Must be within your control
> - Must address a Systemic Pattern identified above
> - Must be observable — you should be able to verify in next month's summary whether it happened
> - "I will try to..." is not a commitment. "I will [action] by [condition]" is.

---

## 🧭 Confidence Delta

> High performers burn out because they track output, not confidence trend.
> Confidence is the early-warning signal for collapse — it degrades before performance does.

**End-of-month self-assessment:**

| Dimension | This Month | Prior Month | Direction |
|---|---|---|---|
| Control over my work | 🟢 In control / 🟡 Partial / 🔴 Reactive | — | ↑/↓/→ |
| Strategic vs tactical | 🟢 Strategic / 🟡 Mixed / 🔴 Tactical | — | ↑/↓/→ |
| Scope clarity | 🟢 Clear / 🟡 Fuzzy / 🔴 Ambiguous | — | ↑/↓/→ |
| Energy headroom | 🟢 Reserve / 🟡 Even / 🔴 Depleted | — | ↑/↓/→ |

**Overall confidence:** 🟢 Growing / 🟡 Stable / 🔴 Declining

**Narrative:** [2 sentences. Not about feelings — about system state. "I feel more reactive than last month because X" is useful. "I feel tired" is not. Frame as a system signal: what does the confidence level tell you about the next 30 days?]

> **How to derive this:**
> - Pull from the EOD reflection sections of weekly summaries
> - Look for language like "felt good about", "overwhelmed", "reactive", "proactive", "clear on what to do next"
> - Aggregate the emotional tone across the month — don't cherry-pick
> - If the prior monthly exists, pull its Confidence Delta for comparison
> - This is a self-report, not a performance score — honesty here is the only signal that matters

---

## 💰 Generation Cost

**Method:** Run `npx ccusage@latest daily --json` before starting and after completing. Diff for today's date.

**Total Cost:** $X.XX

**Breakdown:**
- **Files Read:** X weekly summaries, 1 prior monthly (if available)
- **Files Written:** 1 monthly summary
- **Cache Usage:** Heavy (system prompts reused)

*Cost measured via `npx ccusage@latest daily --json` diff*
```

---

## Invocation

```
/work-os-monthly
```

---

## Quality Bar

**Structural rules:**
- Executive Summary must stand alone — a director should understand the month without reading further
- Narrative Check (Step 0) must be answered before writing — it sets the tone for the entire document
- If it reads like 4 weekly summaries stapled together, it is wrong — synthesize, don't aggregate
- Signal > completeness: a shorter monthly with high-density signal beats a long one with padding

**Section-specific rules:**
- Time allocation gaps > 10% must appear in Systemic Patterns — do not leave them floating
- Firefighting must be extracted separately from coordination, not rolled in as "coordination overhead"
- Systemic Patterns must be genuinely recurring (≥ 3 weeks), not relabeled weekly misses
- Drag Log "Fix attempted?" must be honest — "noted in weekly" is NOT a fix attempt
- Ceiling Raise Index: only Permanent and Durable items count — one-time wins don't compound
- Delegation Score must include missed-delegation items, not only successful delegations
- AI Leverage Quality: default to "Boilerplate" if unclear — avoid inflating the quality signal
- Kill List must be non-empty unless the month genuinely had no subtractions — flag if empty
- Confidence Delta must be derived from EOD reflections, not guessed — if no reflection data, say so
- Promotion Readiness Gaps section must be honest — promo packets without gaps signal low self-awareness
- Stress Attribution: "self-inflicted overcommitment" > 30% is a senior maturity signal — name it directly
