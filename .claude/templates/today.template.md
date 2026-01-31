---
description: Generate a daily work brief from synced work-os raw Markdown
model: opus
---

# Generate Daily Brief

You are generating a concise daily work brief in Markdown from work-os raw Markdown data.

You will read raw Markdown files matching **today's date only** from:

Raw input directory: {{WORK_OS_DIR}}

```
{{WORK_OS_DIR}}/raw/{TODAY-DATE}-{TIME_STAMP}.md
```

There may be multiple files for today (different sync timestamps) — read all of them. Do NOT read files from previous days.

Each file contains semi-structured activity logs and task information from sources such as Slack and GitHub.

You must extract actionable items, ownership, URLs, risks, and context from this Markdown input.

---

## Strict Rules

- Output ONLY valid Markdown
- Target output ≤ 1,500 tokens (hard max 2,000)
- No prose explanations
- Use checkboxes for actionable items
- Include clickable Markdown links when URLs exist
- Summarize aggressively — never quote long messages
- Break down complex tasks with sub-bullets when needed
- Use horizontal separators (---) between major sections
- Group context items by topic with ### headers

---

## Classification Rules

| Category | Rule |
|------------|------|
| Must Do | Directly owned by Shubham Singh (author, assignee, or explicitly requested). Break down into sub-tasks with validation steps. |
| Release-Critical PRs | PRs that block an upcoming release/launch discussed in Slack. Include release name/date and PR status. |
| Reviews | PR reviews explicitly pending by Shubham. Include time pending and requester. |
| Follow-ups | Items waiting on others, scheduled, or blocked. Specify who/what is blocking. |
| Context | Discussions Shubham is mentioned in but not owner. Group by topic. |
| Learning | Patterns, insights, or process gaps observed from today's activities. Actionable observations only. |

### PR Review Status Detection (CRITICAL)

**Before listing any PR as "pending review", check the GitHub data in raw files for review indicators:**

1. Look for `Reviews:` section under each PR entry
2. If `{{USERNAME}}:Commented` or `{{USERNAME}}:Approved` appears → PR is **already reviewed**
3. If `Review Comments:` section contains comments from `{{USERNAME}}` → PR is **already reviewed**

**Classification:**
- **Already reviewed** → Mark as `[x]` and status "reviewed, waiting on @author"
- **Not yet reviewed** → Mark as `[ ]` and status "pending ~Xd (requested by @person)"

**Example raw data indicating PR is already reviewed:**
```
🔀 [GITHUB] feat: CE <> management type support
      Reviews:
      - {{USERNAME}}:Commented

      Review Comments:
      - {{USERNAME}} (file.ts): comment text...
```

This PR should appear as:
```markdown
- [x] [PR #121: CE management type](url) — reviewed, waiting on @author
```

**NOT as:**
```markdown
- [ ] [PR #121: CE management type](url) — pending ~6d (requested by @author)
```

### Release Detection Logic

When processing raw data, look for signals indicating an upcoming release/launch:
- Slack messages mentioning "release", "launch", "go live", "deploy to prod", "ship", "target Friday" etc.
- Timeline references like "by EOD", "tomorrow", "this week", "Friday release"
- Testing discussions that indicate pre-release validation

For each detected release:
1. Identify all related open PRs from GitHub data (by repo, author, or Slack mentions)
2. Cross-reference PR status (open, needs review, approved, merged)
3. Flag any PRs that are NOT yet merged as release blockers

---

## Output Structure

Generate Markdown using this exact structure:

```markdown
# 🗓️ Daily Brief — YYYY-MM-DD

## 🎯 Today's Top 3 Outcomes
- [ ] [Most important outcome with specific deliverable]
- [ ] [Second priority with specific deliverable]
- [ ] [Third priority with specific deliverable]

---

## 🔥 Must Do Today
- [ ] [Task with specific context] — [Slack/PR link]
  - [ ] [Sub-task detail if needed]
  - [ ] [Another sub-task detail]
- [ ] [Another main task] — [link]

---

## 🚀 Release-Critical PRs

### [Release Name] — Target: [Date/Timeline]
| PR | Status | Owner | Blocker? |
|----|--------|-------|----------|
| [PR title](url) | Needs Review / Approved / Open | @author | Yes/No |

---

## 👀 Reviews / Approvals

### Pending Review
- [ ] [PR title](url) — pending ~Xh/Xd (requested by @person)

### Reviewed (Waiting on Author)
- [x] [PR title](url) — reviewed, waiting on @author

---

## 📅 Follow-ups / Waiting
- [ ] [Item description] — [Slack link]

---

## ⚠️ Risks / Blockers
- [Risk description with specific impact]
- [Another blocker with dependency info]

---

## 🧠 Context — Awareness Only

### [Topic Category 1]
- [Discussion summary] — [Slack/link]

### [Topic Category 2]
- [Discussion summary] — [Slack/link]

---

## ✍️ Learning / Improvements
- [Insight or pattern observed from today's work]
- [Process improvement opportunity]

---

## 🪞 End of Day Reflection
(Free-form notes, voice dump, thoughts — no structure required)

---

## 💰 Generation Cost

**Input tokens:** [X tokens]
**Output tokens:** [Y tokens]
**Approximate cost:** $[Z.ZZ]

*Based on Claude Opus pricing: $15/M input tokens, $75/M output tokens*
```

---

## Post-Processing Rules

1. Always extract URLs into Markdown links
2. Merge duplicates aggressively
3. Remove empty sections completely (including headers)
4. Apply stable sorting:
   - Must Do → highest impact first
   - Reviews → oldest pending first
   - Follow-ups → oldest waiting first
   - Context → group by topic, most relevant first
5. For "Must Do Today":
   - Break down complex tasks into sub-bullets with [ ] checkboxes
   - Include specific links to Slack threads or PRs for context
   - Add validation steps or edge cases as sub-items
6. For "Release-Critical PRs":
   - Only include this section if a release/launch is mentioned in Slack messages
   - Group PRs by release/project name
   - Include ALL related open PRs from GitHub data, not just those owned by Shubham
   - Mark "Blocker? Yes" for any PR that is not yet merged but required for release
   - Include PR status: Open, Needs Review, Approved, Changes Requested
   - Link to relevant Slack thread discussing the release timeline
7. For "Context — Awareness Only":
   - Group related items under ### topic headers
   - Use descriptive topic names (e.g., "ILF Integration", "Clusters v2 Planning")
8. For "Learning / Improvements":
   - Extract insights about process gaps, documentation needs, or recurring patterns
   - Focus on actionable observations
9. For "Reviews / Approvals":
   - **CRITICAL:** Check GitHub data for `{{USERNAME}}:Commented` or review comments before classifying
   - PRs with existing review comments from {{USERNAME}} → "reviewed, waiting on @author"
   - PRs without review from {{USERNAME}} → "pending ~Xd (requested by @person)"
   - Split into "Pending Review" and "Reviewed (Waiting on Author)" subsections
10. Prefer clarity over completeness
11. Return ONLY the Markdown content

---

## Process Steps

Execute in this exact order:

### Step 1: Archive Existing Brief

Before anything else, check if `today.md` already exists:

1. If `{{WORK_OS_DIR}}/today.md` exists:
   - Move its contents to: `{{WORK_OS_DIR}}/archive/{YESTERDAY-DATE}.md`
   - Use yesterday's date in `YYYY-MM-DD` format
2. Create the archive directory if it doesn't exist

### Step 2: Read Archive History (Last 7 Days)

Read all archived briefs from the last 7 days:

```
{{WORK_OS_DIR}}/archive/{DATE}.md
```

For each archived brief, identify:
- Uncompleted tasks (checkboxes still marked `- [ ]`)
- Tasks that were in "Likely Carryovers" section
- High priority items that remain open

### Step 3: Read Today's Raw Data

Read ONLY raw Markdown files matching today's date pattern:

```
{{WORK_OS_DIR}}/raw/{TODAY-DATE}-*.md
```

For example, if today is 2026-01-23, read only files like:
- `2026-01-23-0943.md`
- `2026-01-23-1430.md`

**IMPORTANT:** Do NOT read files from previous days. There may be multiple files for today (different timestamps), read all of them. Ignore any files that don't match today's date prefix.

### Step 4: Generate Brief with Carryovers

When generating the new brief:
- Merge incomplete tasks from archives with today's new tasks
- Prioritize carryover tasks based on age and importance
- Deduplicate tasks that appear in both raw data and archives

### Step 5: Carryover Detection

Detect carryover items by matching today's tasks with yesterday's daily brief:

1. For each carryover item include:
   - How many days it has been open (if inferable from archive history)
   - A short reason: `blocked`, `waiting`, `deprioritized`, or `unknown`

2. Add a one-line summary at the top of the Carryovers section:
   ```
   [X items carried over, oldest Y days, Z blocked]
   ```

Example carryover format:
```markdown
## 🔁 Likely Carryovers

3 items carried over, oldest 4 days, 1 blocked

- [ ] [Fix login bug](url) — 4 days, blocked on backend team
- [ ] [Update docs](url) — 2 days, deprioritized
- [ ] [Review PR #123](url) — 1 day, waiting on author
```

---

## Output Location

Write the final brief to:

```
{{WORK_OS_DIR}}/today.md
```

IMPORTANT: Ensure horizontal separators (---) are placed between all major sections as shown in the output structure.

---

## Token Usage and Cost Tracking

At the very end of the generated brief, include a "Generation Cost" section with:
- Input tokens used for this generation
- Output tokens generated
- Approximate cost calculation

Cost calculation formula:
- Input cost = (Input tokens / 1,000,000) × $15
- Output cost = (Output tokens / 1,000,000) × $75
- Total cost = Input cost + Output cost

The token counts should reflect the actual usage for generating the daily brief. Look for token usage information in the system messages or tool results to get accurate counts.

---

## Learning / Improvements Section Guidelines

Extract insights from the raw data that indicate:
- Documentation gaps (e.g., "API contract discussions highlight need for stricter schema validation earlier")
- Process improvements (e.g., "Slack-based clarifications indicate documentation gaps in ILF flows")
- Risk patterns (e.g., "Release risk improves when backend availability is surfaced earlier")
- Communication bottlenecks
- Technical debt signals

Focus on actionable, non-obvious observations. Avoid generic statements.

---

## Fallback Behavior

If no actionable tasks are detected, generate a minimal brief containing:

- Header (`# 🗓️ Daily Brief — YYYY-MM-DD`)
- End of Day Reflection section only

---

## Invocation

```
/work-os-today
```

---

## Preconditions

- Raw Markdown files must already exist in: `{{WORK_OS_DIR}}/raw/`
- Input may contain duplicated threads, quoted replies, timestamps, and noise
- You must normalize and summarize this data

---

## Follow-ups Source (READ-ONLY)

Read follow-ups from:

```
{{WORK_OS_DIR}}/follow-ups.md
```

Rules:
- Read ONLY items under `## Active`
- Do NOT modify this file
- Do NOT create or resolve follow-ups here
- Treat this file as the single source of truth

---

## ⏳ Follow-ups & Waiting (From Ledger)

- [ ] [Follow-up title] — waiting on <person/system> — <age in days>
  [[follow-ups#Active|source]]

Rules:
- Populate ONLY from `follow-ups.md → ## Active`
- Sort by oldest `Since` date first
- Max 5 items
- Do NOT restate detailed context
- Each item MUST backlink using `[[follow-ups#Active|source]]`
- Omit this section entirely if there are no active follow-ups

---

## Addendum — Follow-up Handling Rules

- Follow-ups shown in this file are **read-only projections**
- The authoritative list lives in `follow-ups.md`
- Prefer ledger-backed follow-ups over ad-hoc daily mentions
- Do NOT duplicate the same follow-up across multiple sections
