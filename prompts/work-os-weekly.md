---
description: Generate a weekly work summary from archived daily briefs with lead-level and manager-readable summary
model: opus
---

# Generate Weekly Summary

You are generating a **high-signal weekly work summary in Markdown** from archived daily briefs.

This is a **retrospective and synthesis task** — not a daily task list, not a raw log.

Optimize for clarity, traceability, and decision-making signal.

---

## 🧭 Lead Summary (Manager-Readable) — MANDATORY

At the very top of the output, generate a **lead-level narrative summary** (5–8 bullets maximum).

This summary should allow a manager to understand the week **without reading anything else**.

It must reflect both:
- **Individual contribution** (shipping, reviews, technical ownership)
- **Team enablement** (unblocking, direction-setting, coordination)

Cover:

- Overall week sentiment *(energizing / stressful / mixed / calm)*
- Delivery health *(on-track / stretched / at risk)*
- Focus distribution *(hands-on execution vs reviews/mentorship vs coordination/firefighting)*
- The most meaningful win
- Any notable stressors, risks, or morale signals
- Oscillation health *(Oscillating / Mixed / Linear — based on after-hours pattern and recovery logged in EOD Reflections)*
- Confidence and outlook for the upcoming week
- Optional: team load or capacity signal (if relevant)

Rules:
- Bullet points only
- No checkboxes
- No emojis
- No task-level detail
- Objective, honest, calm tone
- Avoid people-performance evaluation; focus on systems, workload, and delivery
- Reads like a **staff / lead engineer weekly update**

For any **non-subjective claim**, include an Obsidian source link (see “Obsidian Source Linking”).

---

## Data Sources (STRICT)

### Primary Source — Archive (Past 7 Days)

Read archived daily briefs from:

```
$WORK_OS_BASE_DIR/archive/daily/{DATE}.md
```

- Look back **7 calendar days**, counting backwards from today
- Some days may be missing — skip silently

### Additional Source — Today (if exists)

**Check today's data based on the current day of the week:**

**If today is Saturday:**
- **Step 1 — Archive today.md:** Move `today.md` (Friday's checklist) into the archive as Friday's file:
  ```
  $WORK_OS_BASE_DIR/today.md
    → $WORK_OS_BASE_DIR/archive/daily/{FRIDAY_DATE}.md
  ```
  where `{FRIDAY_DATE}` = today minus 1 day (YYYY-MM-DD format).
  Only move if `today.md` exists and `archive/daily/{FRIDAY_DATE}.md` does not already exist.

- **Step 2 — Read Friday data:** The 7-day archive loop will naturally pick up `archive/daily/{FRIDAY_DATE}.md` as part of the past week. No separate read needed.

- **Step 3 — Read Saturday sync:** Additionally read the Saturday archive sync for Friday's work log.
  **Use Bash ls to discover files** (do NOT use Glob — it cannot expand `~` and silently returns nothing):
  ```bash
  ls $WORK_OS_BASE_DIR/raw/{SATURDAY_DATE}/
  ```
  Then read each `sync-*.md` found via the Read tool using its full absolute path:
  ```
  $WORK_OS_BASE_DIR/raw/{SATURDAY_DATE}/sync-HHMM.md
  ```
  **Note:** Sync files live under `raw/`, NOT `archive/`. This is the correct path.
  This is a manual sync taken on Saturday containing Friday's completed work. Merge it with `archive/daily/{FRIDAY_DATE}.md` as a single Friday data source — do not treat it as a separate day.

**If today is any other day:**
- Check if `today.md` exists and read it if present:
  ```
  $WORK_OS_BASE_DIR/today.md
  ```
- Today's brief may not be archived yet, so always check for it
- If the file doesn't exist, skip silently (day off)

Do NOT read:
- raw work-os files
- anything older than 7 days

---

## Obsidian Source Linking (MANDATORY)

All highlighted or inferred insights (wins, misses, risks, releases, learnings, and factual Lead Summary bullets)
must include a **clickable Obsidian link** to their source.

### Preferred Link Formats

1. **Block reference (best, if available)**

```markdown
[[2026-01-26#^some-block-id|source]]
```

2. **Heading reference**

```markdown
[[2026-01-26#🔥 Must Do Today|source]]
```

3. **File reference with line context (fallback)**

```markdown
[[2026-01-26|source]] (lines ~42–58)
```

Rules:
- Append source links at the **end of the bullet**
- Prefer block references when they exist
- Do NOT invent block IDs or headings
- Line numbers are contextual only (Obsidian does not support line anchors)
- If a claim cannot be traced, omit it
- **PRs and Slack threads must be hyperlinks** (e.g., `[PR #123](https://github.com/...)`, `[Slack thread](https://{your-workspace}.slack.com/...)`)

### ⚠️ Wiki-Links in Table Cells — CRITICAL

The `|` character in `[[YYYY-MM-DD|source]]` **breaks Markdown tables** — it is interpreted as a column separator.

**In ALL table cells, always escape the pipe:**
```markdown
[[2026-01-26\|source]]   ✅ correct — renders as a link in Obsidian
[[2026-01-26|source]]    ❌ breaks the table
```

This applies to every table in the output (Reviews, Coding, AI stats, etc.). Never use an unescaped `|` inside `[[...]]` within a table cell.

---

## AI Stats Aggregation

When reading daily briefs, specifically look for the `## 🤖 AI Usage Statistics` section.

1. **Extract LOC data:** Parse the `Total LOC`, `AI LOC`, and `Human LOC` numbers from each day.
2. **Aggregate LOC:**
   - Sum all `Total LOC`, `AI LOC`, `Human LOC`
   - `Weekly AI %` = `(Total AI / Total Weekly LOC) * 100`
3. **Extract Direction data** from each day's `## 🧠 AI Direction Log` section:
   - Sum all prompt counts across all days and projects → `Weekly Prompts`
   - Sum all distinct session counts → `Weekly Sessions`
   - Collect all providers seen → `Providers`
   - Per-day prompt count → used for daily comparison table
4. **Compute weekly Direction Effort label:**
   - 🔴 High: total weekly prompts > 60 OR any single day > 15
   - 🟡 Moderate: total weekly prompts 25–60
   - 🟢 Light: total weekly prompts < 25
5. **Compute direction bar:**
   - 🔴 High: `[████████░░]`
   - 🟡 Moderate: `[█████░░░░░]`
   - 🟢 Light: `[███░░░░░░░]`
6. **Format:**
   - Generate a new 10-block AI LOC progress bar based on weekly AI %.
   - Show Direction Effort label + bars immediately after the LOC summary line.
   - If no AI stats found in any daily brief, omit this section entirely.
   - If no Direction Log data found, omit Direction Effort line and direction bar.

## AI Trend Generation (Verified Daily Data)

1. **Use Aggregated Daily Data:** You have already read the `Total LOC`, `AI LOC`, and `Human LOC` for each available day in the "AI Stats Aggregation" step.
2. **Sort:** Ensure data is sorted by date (Monday → Sunday).
3. **Generate Comparison Table:**
   - Create a table with columns: Date | 🤖 AI | 👤 Human | Winner | AI %
   - For each day, compare AI vs Human LOC
   - **Bold** the larger value in each row
   - Winner column: "AI" if AI > Human, "Human" if Human > AI, "~Tie" if within 5%
   - Calculate AI % = (AI LOC / Total LOC) * 100
   - Add a trend summary below the table describing the weekly pattern

---

## Output Location (MANDATORY)

Write the final weekly summary to:

```
$WORK_OS_BASE_DIR/archive/weekly/{START_DATE}-{END_DATE}.md
```

Filename rules:
- Format: `YYYY-MM-DD_YYYY-MM-DD.md`
- Represents `START_DATE → END_DATE` (inclusive)
- One file per week
- Never overwrite an existing file

---

## Output Structure (EXACT)

```markdown
# 📊 Weekly Summary — {START_DATE} → {END_DATE}

## 🧭 Lead Summary
- **Sentiment:** [Energy level, one line]
- **Delivery:** [On-track / stretched / at risk, brief]
- **Focus:** [Execution vs reviews vs coordination]
- **Key win:** [Single most impactful outcome]
- **Key stressor:** [Main challenge or risk]
- **Outlook:** [Confidence level, next focus areas]

> **Rule:** If a bullet needs a source link, it belongs outside the Lead Summary.
> No dates, PR numbers, or technical details here. Compress for executive scan.

> **Terminology (be consistent for long-term scanning):**
> - **Delivery** → outcomes
> - **Execution** → effort
> - **Shipped** → completed and merged

---

## 📊 Weekly Snapshot
- **Reviews:** X completed, [queue status]
- **PRs authored:** X (Y merged)
- **After-hours:** X nights (dates if any)
- **Carryovers:** X
- **Recovery:** [Oscillating 🟢 / Mixed 🟡 / Linear 🔴] — X after-hours nights, carryovers [stable/rising/falling]
- **Trend:** [spike/steady/light] [when it shifted]
- **Confidence:** 🟢/🟡/🔴 [brief reason]

> Quick metrics for week-over-week comparison.
> Confidence: 🟢 high | 🟡 mixed | 🔴 risk

---

## 🔄 Oscillation Health

**Pattern:** Linear 🔴 / Mixed 🟡 / Oscillating 🟢
*Oscillating = 3+/5 daily protocols logged + ≤ 1 after-hours night · Linear = 0 protocols + 3+ consecutive after-hours nights*

**Load signals:**
- After-hours nights: X (dates)
- Carryover peak: X items ([day])
- P0 spikes: [days with heavy P0 load — or "none"]

**Verdict:** [One-line assessment — e.g. "Mostly linear — heavy delivery week with 3 consecutive after-hours nights. Prioritise recovery before next sprint."]

**Next week:**
- [ ] [Recovery nudge — e.g. "Block recovery time early in the week — load was 🔴 this week"]

> **Detection rules (work signals only):**
> - Oscillating 🟢: ≤ 1 after-hours night + carryover count stable or falling
> - Mixed 🟡: 2 after-hours nights OR carryover count rising
> - Linear 🔴: 3+ consecutive after-hours nights OR carryover count spiking across the week
>
> Source: after-hours count and carryover data from daily archives only. Do not infer recovery activities.

---

## ✅ Wins / Shipped

- [Release-level win — major impact]
  [[YYYY-MM-DD|source]]

- [Feature-level win — medium impact]
  [[YYYY-MM-DD|source]]

- [Maintenance/tooling win — smaller impact]
  [[YYYY-MM-DD|source]]

> **Ordering by implicit weight** (no headers needed, just blank line separation):
> 1. **Release-level** — shipped releases, critical fixes, major milestones
> 2. **Feature-level** — new features, API contracts, process improvements
> 3. **Maintenance/tooling** — bug fixes, dev tooling, cleanup

> **What qualifies as a Win:**
> - Shipped features, bug fixes, or releases with tangible user/team impact
> - Completed technical initiatives (e.g., API contracts finalized, tooling shipped)
> - Process improvements adopted by the team
>
> **What is NOT a Win:**
> - Offloading work to someone else (e.g., "handed off X to Y")
> - Merely completing routine tasks
> - Items with no personal ownership or attachment

---

## ❌ Misses / Slippage
- **Delivery slippage:** [Release/feature that missed target date + why]
  [[YYYY-MM-DD|source]]
- **Process lag (author):** [PR waiting on author response]
  [[YYYY-MM-DD|source]]
- **Process lag (cross-team):** [Item blocked on another team/person]
  [[YYYY-MM-DD|source]]

> **Categorize each miss** to prevent misinterpretation:
> - **Delivery slippage** — missed target dates, late releases
> - **Process lag (author)** — PRs waiting on author changes
> - **Process lag (cross-team)** — blocked on other teams/dependencies

---

## 🔁 Carryovers (My Plate Next Week)
- [ ] [Action I need to take]
  [[YYYY-MM-DD|source]]

> **Only include items that need MY action next week.**
> Exclude items that are:
> - Already listed in Misses (waiting on others)
> - Reviewed and waiting on author
> - Not actively blocking anything
>
> Think: "What will still be on my plate next week?"

---

## 🚀 Releases & Launches
### [Release Name]
- Target: [date/timeline]
- Status: Shipped / Partial / Blocked
- Key PRs:
  - [PR title](url) — status  
    [[YYYY-MM-DD|source]]

---

## 👀 Reviews & Throughput
- Reviews completed: X
- [Narrative insight about review load/timing] — [source](YYYY-MM-DD.md#Section)
- [Insight about bottlenecks or latency drivers] — [source](YYYY-MM-DD.md#Section)
- [Systemic observation if any]

| PR | Repo | Description | Source |
|----|------|-------------|--------|
| [#NNN](url) | repo-name | Brief description (Xd) | [source](YYYY-MM-DD.md#Reviews%20/%20Approvals) |

> **Summary bullets should answer:**
> - Was there a spike or steady flow?
> - What drove any delays — reviewer or author?
> - Any systemic issues detected?
>
> **Table is appendix-style** — readers can stop at summary if needed.
> Add short timing for notable delays: `(6d)` or `(9d, author)` if waiting on author.

---

## 💻 Coding / PRs Authored
- PRs authored: X | Merged: Y
- **Impact:** [One sentence connecting PRs to outcomes]

| PR | Repo | Status | Description | Source |
|----|------|--------|-------------|--------|
| [#NNN](url) | repo-name | Merged/Open | Brief description | [source](YYYY-MM-DD.md#Section%20Name) |

> **Note:** In table cells, always escape the pipe in wiki-links: `[[YYYY-MM-DD\|source]]` not `[[YYYY-MM-DD|source]]`. The unescaped `|` breaks the table. URL-encode spaces as `%20` in heading anchors.

> **How to identify authored PRs:**
> Look for PRs in daily briefs under "Own PRs" sections, or PRs where the user is explicitly the author (not reviewer).
> Only include PRs where the user wrote code, not PRs they reviewed.

---

## 🤖 Week's AI Usage Statistics

**Total LOC:** [Sum Total] | **AI:** [Sum AI] ([AI%]%) | **Human:** [Sum Human] ([Human%]%)
**Direction Effort:** [🟢 Light / 🟡 Moderate / 🔴 High] ([N] prompts · [N] sessions · [providers])

`[Progress Bar 10 blocks]` [AI%]% AI Generated
`[Progress Bar 10 blocks]` ~[Light/Moderate/High] human direction intensity

**Correlation:** [One line — e.g. "High AI LOC + high prompts = AI as fast typist, not autonomous" or "High AI LOC + low prompts = genuine leverage this week"]

### 📈 Daily AI vs Human Comparison

| Date | 🤖 AI | 👤 Human | Prompts | Winner | AI % |
|------|-------|----------|---------|--------|------|
| MM-DD | X,XXX | X,XXX | [N] | AI/Human/~Tie | XX% |

📊 **Trend:** [Narrative of AI vs Human LOC pattern + direction effort pattern — when did AI spike? When did prompt count peak? Any days where high AI LOC but low prompts = genuine leverage?]

> **How to populate:**
> - Bold the larger LOC number in each row
> - Winner: "AI" if AI > Human, "Human" if Human > AI, "~Tie" if within 5%
> - Prompts: sum of prompt count from that day's AI Direction Log (0 if no data)
> - Trend: combine LOC narrative with direction effort narrative — the two together tell the real story

---

## ⚠️ Risks & Blockers

**Ongoing:**
- [Risk still relevant going forward]
  [[YYYY-MM-DD|source]]

**Resolved (lessons captured):**
- [Historical risk — brief lesson]
  [[YYYY-MM-DD|source]]

> Separate ongoing risks from resolved ones to avoid sounding reactive.

---

## 🧠 Key Learnings (Reusable)
- [Short, generalizable, actionable insight]
  [[YYYY-MM-DD|source]]

> **Key Learnings** = reusable process improvements (could apply to future weeks)
> Keep them short and actionable. No narrative context.

---

## 🪞 Weekly Reflection
(Emotional, contextual, narrative — how the week felt, personal observations)

> **Weekly Reflection** = feelings, energy, personal narrative
> No actionable insights here — those belong in Key Learnings.
>
> **Framing tip:** Reframe personal strain as system feedback.
> - Instead of: "The 3 AM push was exhausting"
> - Write: "The 3 AM push was necessary for delivery, but not a sustainable release pattern"

---

## 🎯 Next Week: What I'll Change
- [Concrete action derived from Key Learnings]
- [Another specific change]
- [Third commitment]

> **This is the self-improvement loop.** This section alone makes the doc promotion-ready.
>
> **Rules:**
> - Max 3 bullets
> - Must be actions, not hopes
> - Must be within your control

---

## 💰 Generation Cost

**Method:** Run `npx ccusage@latest daily --json` before starting and after completing all work. Calculate diff for today's date.

**Total Cost:** $X.XX

**Breakdown:**
- **Files Read:** N daily briefs from archive
- **Files Written:** 1 weekly summary
- **Cache Usage:** Heavy (system prompts reused)

*Focus on total cost as primary metric. Token breakdowns may not reflect file content written via tools.*
```

---

## Invocation

```
/work-os-weekly
```

---

## Quality Bar

- Lead Summary must stand on its own
- Synthesis > aggregation
- Claims must be traceable
- No daily-task noise
- If it reads like a TODO list, it is incorrect
- Signal > completeness
