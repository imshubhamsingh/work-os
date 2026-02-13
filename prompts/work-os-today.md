---
description: Generate a daily work brief from synced work-os raw Markdown
model: opus
---

# Generate Daily Brief

You are generating a concise daily work brief in Markdown from work-os raw Markdown data.

You will read raw Markdown files from today's date folder:

```
$WORK_OS_BASE_DIR/raw/{TODAY-DATE}/sync-*.md
```

**Environment Variables:**
- `$WORK_OS_BASE_DIR`: Base directory for work-os data
  - Example: `~/Projects/obsidian/work/00-work-os`

**New File Structure:**
- Date folders: `raw/YYYY-MM-DD/`
- Sync files: `sync-HHMM.md` (24-hour format)
- Example: `raw/2026-02-08/sync-0943.md`, `raw/2026-02-08/sync-1430.md`
There may be multiple sync files for today (different timestamps) — read all `sync-*.md` files in today's folder. Do NOT read files from previous date folders.

Each file contains semi-structured activity logs and task information from sources such as Slack, Jira and GitHub.

---

## Strict Rules

- Output ONLY valid Markdown
- Target output ≤ 1,500 tokens (hard max 2,000)
- No prose explanations
- Use checkboxes for actionable items
- Include clickable Markdown links when URLs exist (Slack, GitHub, Jira)
- Summarize aggressively — never quote long messages
- Break down complex tasks with sub-bullets when needed
- Use horizontal separators (---) between major sections
- Group context items by topic with ### headers
- **Delete/Group DM Filter:** Extract ONLY work-related tasks or critical context. Ignore casual chatter and resolved scheduling.
- **Ack Reaction:** Any message reacted with your acknowledgment emoji **by you** indicates explicit acknowledgement and must be treated as a "Must Do" item regardless of other filters.
  - Configure via `$ACK_REACTION` (Example: `:ack:`)
- **Crucial Items:** Place extremely important items in "Must Do" and **bold the description**.

---

## Classification Rules

| Category | Rule |
|------------|------|
| Must Do | Directly owned by you (author, assignee, explicitly requested, or marked with `$ACK_REACTION` **by you**). **Bold description if extremely/crucially important.** Break down into sub-tasks with validation steps. Link to Jira ticket when the task corresponds to one. |
| Release-Critical PRs | PRs that block an upcoming release/launch discussed in Slack. Include release name/date and PR status. Link related Jira epic/ticket when present. |
| Reviews | PR reviews explicitly pending by you. **Check GitHub data for review status first.** Include time pending and requester. Add Jira link if PR is linked to a ticket. |
| Follow-ups | Items waiting on others, scheduled, or blocked. Specify who/what is blocking. Include Jira link when the follow-up maps to a Jira task. |
| Context | Discussions you are mentioned in but not owner. Group by topic. Attach Jira link when the discussion relates to a known ticket. |
| Learning | Patterns, insights, or process gaps observed from today's activities. Actionable observations only. |

### PR Review Status Detection (CRITICAL)

**Before listing any PR as "pending review", check the GitHub data in raw files for review indicators:**

1. Look for `Reviews:` section under each PR entry
2. If `$GITHUB_USERNAME:Commented` or `$GITHUB_USERNAME:Approved` appears → PR is **already reviewed**
3. If `Review Comments:` section contains comments from `$GITHUB_USERNAME` → PR is **already reviewed**

**Environment Variables:**
- `$GITHUB_USERNAME`: Your GitHub username (Example: `imshubhamsingh`)

**Classification:**
- **Already reviewed** → Mark as `[x]` and status "reviewed, waiting on @author"
- **Not yet reviewed** → Mark as `[ ]` and status "pending ~Xd (requested by @person)"

**Example raw data indicating PR is already reviewed:**
```
🔀 [GITHUB] feat: new feature support
      Reviews:
      - $GITHUB_USERNAME:Commented

      Review Comments:
      - $GITHUB_USERNAME (file.ts): comment text...
```

This PR should appear as:
```markdown
- [x] [PR #121: new feature](url) — reviewed, waiting on @author
```

**NOT as:**
```markdown
- [ ] [PR #121: new feature](url) — pending ~6d (requested by @author)
```

### Jira

- **Actionable:** Extract tasks, ownership, status from Jira like other sources; use for Must Do, Follow-ups, Context. Classify by Status and Assigned to you.
- **Correlation:** Same work in Jira and Slack/GitHub (key, topic, epic) → one item with Jira link. If Slack or PR mentions a key, include that ticket's link.
- **Mismatch (easy to miss):** Jira not Done (In Review, In Testing, Ready for QA) but PR merged or Slack/GitHub say resolved → surface as Follow-up: *Update Jira: [KEY](url) — PR merged; ticket still In Review*.
- **Links:** Use full URL from dump (last line of block). Output as `[KEY: Short title](url)`.

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
- [ ] [Task with specific context] — [Slack/PR/Jira link; provide all available links]
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

## 📋 Jira Status Mismatch
- [ ] Update Jira: [KEY](url) — PR merged / resolved in Slack; ticket still In Review *(only when Jira not Done but Slack/GitHub show work resolved)*

---

## 👀 Reviews / Approvals

### Pending Review
- [ ] [PR title](url) — pending ~Xh/Xd (requested by @person)

### Reviewed (Waiting on Author)
- [x] [PR title](url) — reviewed, waiting on @author

---

## 🔁 Likely Carryovers

**[X items carried over, oldest Y days, Z blocked]**

- [ ] [Task description] — X days, reason
- [ ] [Another task] — X days, reason

---

## ⏳ Follow-ups & Waiting (From Ledger)
- [ ] [Follow-up title] — waiting on <person/system> — <age in days>
  [[follow-ups#Active|source]]

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

## 🤖 AI Usage Statistics

**Total LOC:** [Total] | **AI:** [AI] ([AI%]%) | **Human:** [Human] ([Human%]%)

`[Progress Bar]` [AI%]% AI Generated

### Breakdown

| PR | AI LOC | Human LOC | % AI |
|----|--------|-----------|------|
| [PR Link](url) | [AI] | [Human] | [AI%] |

---

## ✍️ Learning / Improvements
- [Insight or pattern observed from today's work]
- [Process improvement opportunity]

---

## 🪞 End of Day Reflection
(Free-form notes, voice dump, thoughts — no structure required)

---

## 💰 Generation Cost

**Total Tokens:** [Total Tokens Used]
- **Input:** [Input Tokens]
- **Output:** [Output Tokens]
- **Cache Create:** [Cache Create Tokens]
- **Cache Read:** [Cache Read Tokens]

**Total Cost:** $[Total Cost]

*Calculated via `npx ccusage` diff*

*Based on Claude Opus pricing: $15/M input tokens, $75/M output tokens*
```

---

## Post-Processing Rules

1. Always extract URLs into Markdown links (Slack, GitHub, Jira).
2. For items that map to a Jira task, include the Jira URL (from raw data or correlation).
3. Merge duplicates aggressively.
4. **Jira mismatch:** When Jira status is not Done (e.g. In Review, In Testing, Ready for QA) but the correlated PR is merged or Slack/GitHub show the work resolved, add an item in the "Jira Status Mismatch" section to update/close the Jira ticket (e.g. "Update Jira: [KEY](url) — PR merged; ticket still In Review"). Omit this section entirely if no mismatch detected.
5. Remove empty sections completely (including headers)
6. Apply stable sorting:
   - Must Do → highest impact first
   - Reviews → oldest pending first
   - Follow-ups → oldest waiting first
   - Context → group by topic, most relevant first
7. For "Must Do Today":
   - Break down complex tasks into sub-bullets with [ ] checkboxes
   - Include specific links to Slack threads, PRs, and Jira tickets when the task maps to one
   - Add validation steps or edge cases as sub-items
8. For "Release-Critical PRs":
   - Only include this section if a release/launch is mentioned in Slack messages
   - Group PRs by release/project name
   - Include ALL related open PRs from GitHub data, not just those owned by you
   - Mark "Blocker? Yes" for any PR that is not yet merged but required for release
   - Include PR status: Open, Needs Review, Approved, Changes Requested
   - Link to relevant Slack thread discussing the release timeline
9. For "Context — Awareness Only":
   - Group related items under ### topic headers
   - Use descriptive topic names (e.g., "Feature X Integration", "Project Y Planning")
10. For "Learning / Improvements":
   - Extract insights about process gaps, documentation needs, or recurring patterns
   - Focus on actionable observations
11. For "Reviews / Approvals":
   - **CRITICAL:** Check GitHub data for `{YOUR_GITHUB_USERNAME}:Commented` or review comments before classifying
   - PRs with existing review comments from you → "reviewed, waiting on @author"
   - PRs without review from you → "pending ~Xd (requested by @person)"
   - Split into "Pending Review" and "Reviewed (Waiting on Author)" subsections
12. Prefer clarity over completeness
13. Return ONLY the Markdown content
14. **AI Usage Statistics:**
   - If "AI Usage Statistics" or "📊 [GITHUB] AI Usage" appears in raw data:
     - Extract Total, AI, and Human LOC counts and percentages.
     - Create a 10-block ASCII progress bar (e.g., `████████░░`) for AI %.
     - Create a table for "PR Breakdown" with columns: PR (Link), AI LOC, Human LOC, % AI.
     - Simplify repo names (e.g., `org/repo-name` → `repo-name`).
     - **Format:**
       ```markdown
       ## 🤖 AI Usage Statistics

       **Total LOC:** 4,377 | **AI:** 3,522 (80.5%) | **Human:** 855 (19.5%)

       `[████████░░]` 80.5% AI Generated

       ### Breakdown
       | PR | AI LOC | Human LOC | % AI |
       |----|--------|-----------|------|
       | [#155 repo-name](url) | 3,418 | 855 | 80% |
       ```

---

## Process Steps

Execute in this exact order:

### Step 0: Track Starting Token Usage

Before beginning any file operations, execute:
```bash
npx ccusage
```

Parse the output table. Locate the **Total** row at the bottom.
Record the values for:
- **Input**
- **Output**
- **Cache Create**
- **Cache Read**
- **Total Tokens**
- **Cost (USD)**

Store these as `START_INPUT`, `START_OUTPUT`, `START_CACHE_CREATE`, `START_CACHE_READ`, `START_TOKENS`, and `START_COST`.

### Step 1: Archive Existing Brief

Before anything else, check if `today.md` already exists:

1. If `$WORK_OS_BASE_DIR/today.md` exists:
   - Move its contents to: `$WORK_OS_BASE_DIR/archive/{YESTERDAY-DATE}.md`
   - Use yesterday's date in `YYYY-MM-DD` format
2. Create the archive directory if it doesn't exist

### Step 2: Read Archive History (Last 7 Days)

Read all archived briefs from the last 7 days:

```
$WORK_OS_BASE_DIR/archive/{DATE}.md
```

For each archived brief, identify:
- Uncompleted tasks (checkboxes still marked `- [ ]`)
- Tasks that were in "Likely Carryovers" section
- High priority items that remain open

### Step 3: Read Today's Raw Data

Read ONLY sync files from today's date folder:

```
$WORK_OS_BASE_DIR/raw/{TODAY-DATE}/sync-*.md
```

**New Structure:**
- Date folder: `raw/YYYY-MM-DD/`
- Sync files: `sync-HHMM.md` (24-hour time format)

For example, if today is 2026-01-23, read files like:
- `raw/2026-01-23/sync-0943.md`
- `raw/2026-01-23/sync-1430.md`
- `raw/2026-01-23/sync-1845.md`

**IMPORTANT:**
- Read ALL `sync-*.md` files in today's date folder
- Do NOT read files from previous date folders (e.g., `raw/2026-01-22/`)

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

### Step 6: Calculate and Add Generation Cost

After generating the complete brief:

1. **Run Usage Check Again:**
   ```bash
   npx ccusage
   ```

2. **Calculate Diff:**
   - Parse the **Total** row again to get the END values.
   - Calculate deltas for all metrics:
     - **Input** = `END_INPUT` - `START_INPUT`
     - **Output** = `END_OUTPUT` - `START_OUTPUT`
     - **Cache Create** = `END_CACHE_CREATE` - `START_CACHE_CREATE`
     - **Cache Read** = `END_CACHE_READ` - `START_CACHE_READ`
     - **Total Tokens** = `END_TOKENS` - `START_TOKENS`
     - **Total Cost** = `END_COST` - `START_COST`

3. **Append Generation Cost section** to the end of `today.md` with the calculated delta values.

---

## Output Location

Write the final brief to:

```
$WORK_OS_BASE_DIR/today.md
```

IMPORTANT: Ensure horizontal separators (---) are placed between all major sections as shown in the output structure.

---

## Token Usage and Cost Tracking

At the very end of the generated brief, include a "## 💰 Generation Cost" section.

### Method

1. Run `npx ccusage` at the **start** of execution.
2. Run `npx ccusage` at the **end** of execution.
3. Calculate the difference in the **Total** row for "Total Tokens" and "Cost (USD)".

### Output Format

```markdown
## 💰 Generation Cost

**Total Tokens:** X,XXX
- **Input:** X,XXX
- **Output:** X,XXX
- **Cache Create:** X,XXX
- **Cache Read:** X,XXX

**Total Cost:** $X.XX

*Calculated via `npx ccusage` diff*
```

---

## Learning / Improvements Section Guidelines

Extract insights from the raw data that indicate:
- Documentation gaps (e.g., "API contract discussions highlight need for stricter schema validation earlier")
- Process improvements (e.g., "Slack-based clarifications indicate documentation gaps in feature flows")
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

- Raw Markdown files must already exist in: `$WORK_OS_BASE_DIR/raw/`
- Input may contain duplicated threads, quoted replies, timestamps, and noise
- You must normalize and summarize this data

---

## Follow-ups Source (READ-ONLY)

Read follow-ups from:

```
$WORK_OS_BASE_DIR/follow-ups.md
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
