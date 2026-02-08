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

For any **non-subjective claim**, include an Obsidian source link (see "Obsidian Source Linking").

---

## Data Sources (STRICT)

### Primary Source — Archive (Past 7 Days)

Read archived daily briefs from:

```
~/Projects/obsidian/work/00-work-os/archive/{DATE}.md
```

- Look back **7 calendar days**, counting backwards from today
- Some days may be missing — skip silently

### Fallback Source — Today

If today's date is **not included** in the archive window, additionally read:

```
~/Projects/obsidian/work/00-work-os/today.md
```

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
- **PRs and Slack threads must be hyperlinks** (e.g., `[PR #123](https://github.com/...)`, `[Slack thread](https://...)`)

---

## AI Stats Aggregation (NEW)

When reading daily briefs, specifically look for the `## 🤖 AI Usage Statistics` section.

1. **Extract Data:** Parse the `Total LOC`, `AI LOC`, and `Human LOC` numbers from each day.
2. **Aggregate:**
   - Sum all `Total LOC`
   - Sum all `AI LOC`
   - Sum all `Human LOC`
3. **Calculate Weekly Stats:**
   - `Weekly AI %` = `(Total AI / Total Weekly LOC) * 100`
   - `Weekly Human %` = `(Total Human / Total Weekly LOC) * 100`
4. **Format:**
   - Generate a new 10-block progress bar based on the *weekly* percentage.
   - Present the summary data (Total, AI, Human stats) in the `## 🤖 Week's AI Usage Statistics` section.
   - **Do NOT** include a per-PR breakdown table.
   - If no AI stats are found in any daily brief, omit this section entirely.

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
~/Projects/obsidian/work/00-work-os/archive/weekly/{START_DATE}-{END_DATE}.md
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
- **Trend:** [spike/steady/light] [when it shifted]
- **Confidence:** 🟢/🟡/🔴 [brief reason]

> Quick metrics for week-over-week comparison.
> Confidence: 🟢 high | 🟡 mixed | 🔴 risk

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

> **Note:** Use standard markdown links (not wiki-links) for table cells. URL-encode spaces as `%20` in heading anchors.

> **How to identify authored PRs:**
> Look for PRs in daily briefs under "Own PRs" sections, or PRs where the user is explicitly the author (not reviewer).
> Only include PRs where the user wrote code, not PRs they reviewed.

---

## 🤖 Week's AI Usage Statistics

**Total LOC:** [Sum Total] | **AI:** [Sum AI] ([New %]%) | **Human:** [Sum Human] ([New %]%)

`[Progress Bar]` [New %]% AI Generated

### 📈 Daily AI vs Human Comparison

| Date | 🤖 AI | 👤 Human | Winner | AI % |
|------|-------|----------|---------|------|
| MM-DD | X,XXX | X,XXX | AI/Human/~Tie | XX% |

📊 **Trend:** [Brief narrative of AI vs Human pattern across the week - when did AI spike? When did it drop? Any notable shifts?]

> **How to populate:**
> - Bold the larger number in each row
> - Winner column: "AI" if AI > Human, "Human" if Human > AI, "~Tie" if within 5%
> - AI % = (AI / Total) * 100
> - Trend: Narrative summary of the weekly pattern (e.g., "AI dominated early week, Human regained control Thu-Fri")

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

**Input tokens:** [X]
**Output tokens:** [Y]
**Approximate cost:** $[Z.ZZ]
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
