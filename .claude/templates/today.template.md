---
description: Generate a daily work brief from synced work-os raw Markdown
model: opus
---

# Generate Daily Brief

You are generating a concise daily work brief in Markdown from work-os raw Markdown data.

You will receive the concatenated contents of all raw Markdown files from:

Raw input directory: {{WORK_OS_DIR}}

```
{{WORK_OS_DIR}}/raw/{YYYY-MM-DD}-{TIME_STAMP}.md
```


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
| Reviews | PR reviews explicitly pending by Shubham. Include time pending and requester. |
| Follow-ups | Items waiting on others, scheduled, or blocked. Specify who/what is blocking. |
| Context | Discussions Shubham is mentioned in but not owner. Group by topic. |
| Learning | Patterns, insights, or process gaps observed from today's activities. Actionable observations only. |

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

## 👀 Reviews / Approvals
- [ ] [PR title](url) — pending ~Xh/Xd (requested by @person)

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
6. For "Context — Awareness Only":
   - Group related items under ### topic headers
   - Use descriptive topic names (e.g., "ILF Integration", "Clusters v2 Planning")
7. For "Learning / Improvements":
   - Extract insights about process gaps, documentation needs, or recurring patterns
   - Focus on actionable observations
8. Prefer clarity over completeness
9. Return ONLY the Markdown content

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

Read all raw Markdown files from:

```
{{WORK_OS_DIR}}/raw/
```

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

