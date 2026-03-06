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

**New Structure:**
- Files are organized in date folders: `raw/YYYY-MM-DD/`
- Each sync creates: `sync-HHMM.md` (24-hour format)
- Example: `raw/2026-02-08/sync-0943.md`, `raw/2026-02-08/sync-1430.md`

There may be multiple sync files for today (different timestamps) — read all `sync-*.md` files in today's folder. Do NOT read files from previous date folders.

Each file contains semi-structured activity logs and task information from sources such as Slack, Jira and Gityour product.

---

## Strict Rules

- Output ONLY valid Markdown
- Target output ≤ 1,500 tokens (hard max 2,000)
- No prose explanations
- Use checkboxes for actionable items
- Include clickable Markdown links when URLs exist (Slack, Gityour product, Jira)
- **SLACK LINKS — ALWAYS USE THE EXACT FULL URL FROM THE RAW SYNC FILE:**
  - **Thread message:** include the full URL with `?thread_ts=` parameter — e.g. `https://{your-workspace}.slack.com/archives/C092RJAL9EW/p1772717406026889?thread_ts=1772710522.703809`
  - **Standalone message:** use the full URL as-is — e.g. `https://{your-workspace}.slack.com/archives/C04H58P2BEX/p1772724460925309`
  - **DM / Group DM:** use the channel-level URL found at the bottom of the message block — e.g. `https://slack.com/archives/D04KZQZ4LGH` — never omit it just because there is no per-message ID
  - **NEVER truncate, shorten, or reconstruct Slack URLs** — copy them verbatim from the raw data. The URL always appears as the last line of a `💬 [SLACK]` block.
- Summarize aggressively — never quote long messages
- Break down complex tasks with sub-bullets when needed
- Use horizontal separators (---) between major sections
- Group context items by topic with ### headers
- **Delete/Group DM Filter:** Extract ONLY work-related tasks or critical context. Ignore casual chatter and resolved scheduling.
- **Ack Reaction:** Any message reacted with `$ACK_REACTION` **by you** indicates explicit acknowledgement and must be treated as a "Must Do" item regardless of other filters.
- **Crucial Items:** Place extremely important items in "Must Do" and **bold the description**.
- **Granola MOM:** Extract action items from Granola meeting notes found in ANY Slack channel (DMs, team channels, group chats). Parse "Action Items" or "Agreed Next Steps" sections. Classify based on Owner. Include meeting title and link to Granola notes.
- **Automated Messages (`»` marker):** Any Slack message, Gityour product comment, or Jira comment starting with `»` was sent automatically via work-os-execute delegation system. These are mechanical follow-ups (PR pings, status updates, etc.) and should be contextualized as automated actions rather than manual interactions.

---

## Cross-Source Correlation

**Semantic Linking:**
Identify when different sources (Slack, Gityour product, Jira, Granola) refer to the same work:
- **Jira keys** mentioned in PR titles, Slack threads, or meeting notes
- **PR numbers** referenced in Jira tickets or Slack discussions
- **Feature/topic names** across multiple sources (e.g., "vendor linking", "Zendesk integration")
- **Meeting decisions** that create tasks in Jira or PRs in Gityour product

**Merge into Single Item:**
When the same work appears across sources, create ONE item with ALL relevant links:
```
- [ ] Implement vendor data availability — [PR #234](github_url) · [PROJ-123](jira_url) · [Tech Sync MOM](granola_url) — P0, target 2w
```

**Status Mismatches:**
Surface when sources disagree (e.g., Jira says "In Progress" but PR is merged, or meeting decided to proceed but Jira not updated)

---

## Classification Rules

| Category | Rule |
|------------|------|
| Must Do | Directly owned by you (author, assignee, explicitly requested, or marked with `$ACK_REACTION` reaction **by you**). **Bold description if extremely/crucially important.** Break down into sub-tasks with validation steps. Link to Jira ticket when the task corresponds to one. |
| Release-Critical PRs | PRs that block an upcoming release/launch discussed in Slack. Include release name/date and PR status. Link related Jira epic/ticket when present. |
| Reviews | PR reviews explicitly pending by you. **Check Gityour product data for review status first.** Include time pending and requester. **CRITICAL: Always include Slack thread link if someone requested review via Slack.** Add Jira link if PR is linked to a ticket. |
| Follow-ups | Items waiting on others, scheduled, or blocked. Specify who/what is blocking. Include Jira link when the follow-up maps to a Jira task. |
| Context | Discussions you is mentioned in but not owner. Group by topic. Attach Jira link when the discussion relates to a known ticket. |
| Learning | Patterns, insights, or process gaps observed from today's activities. Actionable observations only. |

### PR Review Status Detection (CRITICAL)

**Before listing any PR as "pending review", check the Gityour product data in raw files for review indicators:**

1. Look for `Reviews:` section under each PR entry
2. If `$GITHUB_USERNAME:Commented` or `$GITHUB_USERNAME:Approved` appears → PR is **already reviewed**
3. If `Review Comments:` section contains comments from `$GITHUB_USERNAME` → PR is **already reviewed**

**Slack Thread Detection (MANDATORY):**
- Search **today's AND yesterday's raw sync files** for Slack messages mentioning the PR number
- Check **all channels**: public channels (#your-project-channel), DMs, group chats
- Common patterns: "Kindly review", "Please review the PR", "@{your-slack-handle}" + PR link, "review this" + PR URL
- **Include channel context in link text:**
  - Public channel: `— [Slack: #channel-name](slack_url)`
  - DM: `— [Slack: DM](slack_url)`
  - Group DM: `— [Slack: group DM](slack_url)`
- Extract channel name from raw data patterns:
  - `💬 [SLACK] Mention in your-project-channel` → `#your-project-channel`
  - `💬 [SLACK] DM between you and Person` → `DM`
  - `💬 [SLACK] Group messaging with:` → `group DM`
- If multiple Slack threads mention the PR, use the most recent one
- **DM priority:** PR review requests in DMs are high priority - always surface them

**Classification:**
- **Already reviewed** → Mark as `[x]` and status "reviewed, waiting on @author"
- **Not yet reviewed** → Mark as `[ ]` and status "pending ~Xd (requested by @person) — [Slack: #channel-name](url)" or "[Slack: DM](url)" or "[Slack: group DM](url)"

**Example formats:**
```markdown
- [ ] [PR #172](url) — ~1d (by @person) — [Slack: #your-project-channel](slack_url)
- [ ] [PR #167](url) — ~6d (by @person) — [Slack: group DM](slack_url)
- [ ] [PR #123](url) — ~2d (by @person) — [Slack: DM](slack_url)
```

**Example raw data indicating PR is already reviewed:**
```
🔀 [GITHUB] feat: CE <> management type support
      Reviews:
      - $GITHUB_USERNAME:Commented

      Review Comments:
      - $GITHUB_USERNAME (file.ts): comment text...
```

This PR should appear as:
```markdown
- [x] [PR #121: CE management type](url) — reviewed, waiting on @author
```

**NOT as:**
```markdown
- [ ] [PR #121: CE management type](url) — pending ~6d (requested by @author)
```

### Jira

- **Actionable:** Extract tasks, ownership, status from Jira like other sources; use for Must Do, Follow-ups, Context. Classify by Status and Assigned: you.
- **Correlation:** See "Cross-Source Correlation" section. Link Jira tickets with related PRs, Slack threads, and meeting notes.
- **Mismatch (easy to miss):** Jira not Done (In Review, In Testing, Ready for QA) but PR merged or Slack/Gityour product say resolved → surface as Follow-up: *Update Jira: [KEY](url) — PR merged; ticket still In Review*.
- **Links:** Use full URL from dump (last line of block). Output as `[KEY: Short title](url)`.

### Granola MOM (Minutes of Meeting)

**Sources:**
- **Direct Sync:** `🎤 [GRANOLA]` entries in sync files
- **Slack-Pasted:** Meeting notes shared in Slack (any channel)

**Extraction (Template-Agnostic):**
Templates vary. Semantically identify:
- Actionable items with owner and timeline (anywhere in content)
- Priority indicators (P0/P1, "Critical", "Blocker")
- Link to full MOM (see link priority below)

**Link Priority:**
1. **Prefer file URL** (`file:///path/to/moms/folder`) - points to local summary + transcript
2. **Fallback to Granola URL** (`https://notes.granola.ai/t/...`) - only if file URL unavailable
3. **Last resort:** Use `via #channel` if no links found

**Classification:**
- you owned → "Must Do Today"
- Others owned → "Follow-ups & Waiting"
- TBD/Team → Skip unless P0

**Output:**
One-line summaries (max 3-5 per meeting, P0 only):
```
- [ ] [Action] — [Meeting MOM](file_url) (w/ @person) — P0, target 2w
- [ ] [Action] — [Meeting](granola_url) — waiting on @person
```

### Release Detection Logic

**CRITICAL:** Always check for release-critical items. Look for:
- Slack messages with "deploy", "release", "Please deploy", "@{your-slack-handle} After [person] deploys"
- Deployment sequences (e.g., "deploy X first, then Y")
- Open PRs mentioned in yesterday's Release-Critical section that are still open today
- Slack threads explicitly requesting deployment from you

**Detection pattern:**
1. Search raw data for: "deploy", "release", "launch", "go live", "ship"
2. Check yesterday's brief for any Release-Critical PRs still open
3. Identify deployment chains (A blocks B)
4. Include ALL open PRs in the release, not just yours

---

## Output Structure

Generate Markdown using this exact structure:

```markdown
# 🗓️ Daily Brief — YYYY-MM-DD

## 🎯 Today's Top 3 Outcomes
- [ ] [Most important outcome with specific deliverable]
- [ ] [Second priority with specific deliverable]
- [ ] [Third priority with specific deliverable]

**IMPORTANT:** If a Top 3 item references a PR, Jira ticket, or Slack thread, include the hyperlink in the description. Examples:
- [ ] Review [PR #172](github_url) and [PR #171](github_url) — clear review backlog
- [ ] Deploy PROJECT v1.3.0 to SOS — [proj-ilf thread](slack_url)
- [ ] Continue Feature X Sprint 2 — [PR #155](github_url) · [plan doc](github_url)

---

## 🔥 Must Do Today
- [ ] [Task with specific context] — [Slack/PR/Jira link; provide all available links]
  - [ ] [Sub-task detail if needed]
  - [ ] [Another sub-task detail]
- [ ] Remove commission/FF fields from vendor creation form — [Meeting MOM](granola_url) — P0, target Mon
- [ ] Implement unified Zendesk API in Payload — [Integration Sync MOM](granola_url) — target EOW
- [ ] [Another main task] — [link]

---

## 🚀 Release-Critical PRs

**IMPORTANT:** This section is MANDATORY if any of these conditions are met:
- Slack messages with "Please deploy", "deploy this", "ready for deployment"
- Yesterday's brief had Release-Critical PRs that are still open today
- Deployment sequences mentioned (e.g., "deploy X first, then Y")
- PRs explicitly blocked on other deployments

If no releases detected, omit this section entirely.

### [Release Name] — Target: [Date/Timeline]
| PR | Status | Owner | Blocker? |
|----|--------|-------|----------|
| [PR title](url) | Needs Review / Approved / Open | @author | Yes/No |

---

## 📋 Jira Status Mismatch
- [ ] Update Jira: [KEY](url) — PR merged / resolved in Slack; ticket still In Review *(only when Jira not Done but Slack/Gityour product show work resolved)*

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
- [ ] Workflow sheet finalization — waiting on @team-member — [Integration Sync MOM](granola_url) — target Wed
- [ ] Domain verification implementation — waiting on @team-member — [Platform MOM](granola_url)

---

## ⚠️ Risks / Blockers
- [Risk description with specific impact]
- [Another blocker with dependency info]

---

## 🔋 Recovery Check

**Load Level:** 🟢 Low / 🟡 Moderate / 🔴 High
*Signals: [list detected signals — e.g. "3 carryovers (4+ days), 1 after-hours session Mon"]*

**Recommendation:** [One line — e.g. "Load is elevated — block recovery time before your next sprint." or "Load is low — good time to push hard today."]

---

## 🧠 Context — Awareness Only

### [Topic Category 1]
- [Discussion summary] — [Slack/link]

### [Meeting: Topic Name]
- [1-2 line critical decision or context] — [Granola MOM](granola_url)

### [Topic Category 2]
- [Discussion summary] — [Slack/link]

---

---

## 🤖 AI Usage Statistics

**Total LOC:** [Total] | **AI:** [AI] ([AI%]%) | **Human:** [Human] ([Human%]%)

**Direction Effort:** [🟢 Light / 🟡 Moderate / 🔴 High] — [N] prompts · [N] sessions · [providers]

`[Progress Bar 10 blocks]` [AI LOC %] AI Generated · [AI LOC] AI / [Human LOC] Human
`[Progress Bar 10 blocks]` ~[Light/Moderate/High] direction · [N] prompts · [N] sessions

**Correlation:** [One line — e.g. "High AI LOC (89%) + 14 prompts = high-direction, not leverage" or "High AI LOC (80%) + 3 prompts = genuine leverage"]

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

*Calculated via `npx ccusage@latest daily --json` diff*

*Based on Claude Opus pricing: $15/M input tokens, $75/M output tokens*
```

---

## Post-Processing Rules

1. **Always use the exact full Slack URL from the raw data — no exceptions.**
   - Each `💬 [SLACK]` block ends with a URL line — that is the canonical link to use.
   - Thread URLs contain `?thread_ts=XXXXXXXXXX.XXXXXX` — keep this parameter, it deep-links to the specific thread.
   - Standalone message URLs are `https://{your-workspace}.slack.com/archives/CXXXXXXXX/pXXXXXXXXXXXXXXX` — use as-is.
   - DM / Group DM URLs are `https://slack.com/archives/DXXXXXXXXXX` (channel-level only, no message ID) — still include this URL; it's the best available link.
   - If a message block has NO URL at all, omit the link rather than guessing.
2. Always extract URLs into Markdown links (Slack, Gityour product, Jira, Granola).
3. **Cross-source correlation:** Identify when multiple sources discuss the same work. Create ONE item with ALL relevant links (PR + Jira + Slack + Granola). Look for: Jira keys in PR titles/Slack, PR numbers in Jira/Slack, shared feature names, meeting decisions creating tasks.
3. **Automated Messages (`»` marker):** When you see messages/comments starting with `»`, these were sent via work-os-execute delegation system. Contextualize appropriately:
   - PR comments with `»`: "automated ping sent to @user"
   - Slack messages with `»`: "automated follow-up posted"
   - Don't treat automated pings as pending actions if they were just sent today
   - Focus on the response or next action needed, not the automated message itself
4. Merge duplicates aggressively across all sources.
4. **Jira mismatch:** When Jira status is not Done (e.g. In Review, In Testing, Ready for QA) but the correlated PR is merged or Slack/Gityour product show the work resolved, add an item in the "Jira Status Mismatch" section to update/close the Jira ticket (e.g. "Update Jira: [KEY](url) — PR merged; ticket still In Review"). Omit this section entirely if no mismatch detected.
5. **Automated delegation actions:** When you see automated messages sent today (identified by `»` marker), don't create new follow-up tasks for them unless there's a specific next action needed. The automated ping itself was the action. Focus on tracking responses or blocked items.
6. Remove empty sections completely (including headers)
7. Apply stable sorting:
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
   - Include ALL related open PRs from Gityour product data, not just those owned by you
   - Mark "Blocker? Yes" for any PR that is not yet merged but required for release
   - Include PR status: Open, Needs Review, Approved, Changes Requested
   - Link to relevant Slack thread discussing the release timeline
9. For "Context — Awareness Only":
   - Group related items under ### topic headers
   - Use descriptive topic names (e.g., "PROJECT Integration", "Clusters v2 Planning")
10. For "Learning / Improvements":
   - Extract insights about process gaps, documentation needs, or recurring patterns
   - Focus on actionable observations
11. For "Reviews / Approvals":
   - **CRITICAL:** Check Gityour product data for `$GITHUB_USERNAME:Commented` or review comments before classifying
   - PRs with existing review comments from $GITHUB_USERNAME → "reviewed, waiting on @author"
   - PRs without review from $GITHUB_USERNAME → "pending ~Xd (requested by @person)"
   - Split into "Pending Review" and "Reviewed (Waiting on Author)" subsections
12. For "Granola MOM Processing":
   - **CRITICAL:** Two formats exist - Direct Sync (structured) and Slack-Pasted (freeform)

   **Direct Sync Format (Primary):**
   - **Detection:** Look for `🎤 [GRANOLA]` entries in sync files
   - **Semantic Extraction:** Don't rely on exact section names - use contextual understanding
   - **Actionables:** Identify tasks/decisions with owner and timeline information (wherever they appear)
   - **Ownership:** Extract who is responsible (may be labeled as Owner, assignee, @mention, or embedded in text)
   - **Priority:** Look for P0/P1, "Immediate", "Critical", or other priority indicators
   - **Links:** Use file path (`file:///...`) or Granola transcript URL for full context
   - **Format:** `[Action] — [Meeting Title MOM](file_path) (w/ attendees) — P0/P1, target X`

   **Slack-Pasted Format (Legacy):**
   - **Detection:** Contextual - meeting-style content with decisions/actions (format varies)
   - **Link Search:** Check pasted content first, then surrounding Slack thread messages
   - **Format Agnostic:** Don't expect specific section names - extract semantically
   - **Source:** Can appear in ANY Slack channel (DMs, #team-channels, group chats)

   **Common Rules (Both Formats):**
   - **Template-agnostic:** MOM templates change - focus on semantic understanding, not field names
   - One line per action item maximum
   - Skip discussion context, open questions (link provides full context)
   - Extract max 3-5 items per meeting (P0/highest priority only)
   - For "Must Do": Owner is you + include target and priority
   - For "Follow-ups": Owner is others + include who you're waiting on
   - For "Context": Only include if critical decision impacts current work
13. Prefer clarity over completeness
14. Return ONLY the Markdown content
15. **Recovery Check:**
   - **Load level detection** (use highest matching tier):
     - 🔴 High: 5+ active carryovers OR 2+ consecutive after-hours nights in archive OR "overwhelmed / firefighting / stretched" in recent EOD Reflections
     - 🟡 Moderate: 3–5 carryovers OR 1 after-hours session this week OR 3+ P0 items stacked
     - 🟢 Low: ≤ 2 carryovers, no after-hours, manageable P0 count
   - **Recommendation:** one sentence calibrated to load level:
     - 🟢 Low: "Load is low — good conditions to push hard today."
     - 🟡 Moderate: "Load is building — consider blocking recovery time today."
     - 🔴 High: "Load is elevated — recovery before the next sprint is important."
   - No specific activities, protocols, or personal routines — keep it work-signal-based only
   - Omit Recovery Check section entirely only if no archive data exists at all
14. **AI Usage Statistics** (merged from Gityour product LOC + `ai-sessions.jsonl`):
   - **LOC data** — from "AI Usage Statistics" or "📊 [GITHUB] AI Usage" in raw sync data:
     - Extract Total, AI, Human LOC counts and percentages
     - Create a 10-block ASCII progress bar for AI %
     - Simplify repo names (e.g., `your-org/your-repo` → `your-repo`)
     - If absent, omit LOC lines
   - **Direction data** — from today's `ai-sessions.jsonl` (snapshot at time of generation):
     - Count prompts (`event == "prompt"`), distinct `session_id`s, providers used
     - Intensity: 🔴 > 15 prompts · 🟡 > 6 · 🟢 ≤ 6
     - Direction bar: 🔴 High = 8 blocks · 🟡 Moderate = 5 blocks · 🟢 Light = 3 blocks
     - If absent, omit Direction Effort lines
   - **Correlation** — cross-reference AI LOC % with prompt count (omit if either is missing):
     - AI LOC > 70% AND prompts > 10 → "high-direction: AI was fast typist, not autonomous"
     - AI LOC > 70% AND prompts ≤ 5 → "genuine leverage: AI ran on clear specs"
     - AI LOC < 40% AND prompts > 10 → "high human effort, low AI output — debugging or exploration"
   - **PR table** — one row per PR, no subheader, no project breakdown table, no prompt log
   - Omit section entirely if both LOC data and `ai-sessions.jsonl` are absent
   - **Format:**
     ```markdown
     ## 🤖 AI Usage Statistics

     **Total LOC:** 4,377 | **AI:** 3,522 (80.5%) | **Human:** 855 (19.5%)

     **Direction Effort:** 🟡 Moderate — 7 prompts · 1 session · claude

     `[████████░░]` 80.5% AI Generated · 3,522 AI / 855 Human
     `[█████░░░░░]` ~Moderate direction · 7 prompts · 1 session

     **Correlation:** High AI LOC (80%) + 7 prompts = moderate-direction; AI doing heavy lifting with light human steering

     | PR | AI LOC | Human LOC | % AI |
     |----|--------|-----------|------|
     | [#155 your-repo](url) | 3,418 | 855 | 80% |
     ```

---

## Process Steps

Execute in this exact order:

### Step 0: Track Starting Token Usage

Before beginning any file operations, execute:
```bash
npx ccusage@latest daily --json
```

Parse the JSON output. Find the object in the `daily` array where the `date` field matches today's date (YYYY-MM-DD).
Extract these values from that object:
- `inputTokens` → `START_INPUT`
- `outputTokens` → `START_OUTPUT`
- `cacheCreationTokens` → `START_CACHE_CREATE`
- `cacheReadTokens` → `START_CACHE_READ`
- `totalTokens` → `START_TOKENS`
- `totalCost` → `START_COST`

### Step 1: Archive Existing Brief

Before anything else, check if `today.md` already exists:

1. If `$WORK_OS_BASE_DIR/today.md` exists:
   - Move its contents to: `$WORK_OS_BASE_DIR/archive/daily/{YESTERDAY-DATE}.md`
   - Use yesterday's date in `YYYY-MM-DD` format
2. Create the archive directory if it doesn't exist

### Step 2: Read Archive History (Last 7 Days)

Read archived briefs from the last 7 days using a **tiered approach** to minimize token usage:

```
$WORK_OS_BASE_DIR/archive/daily/{DATE}.md
```

**Tiered reading strategy:**
- **Yesterday's brief (YYYY-MM-DD)** — READ IN FULL (no line limit). This is critical for detecting:
  - Release-Critical PRs that are still open today
  - Carryover age tracking
  - Context continuity
- **2–7 days ago** — read only the **first 50 lines** of each archive file. The top sections (Top 3, Must Do, Carryovers) contain all information needed for carryover detection and age tracking.

**CRITICAL for Release Detection:**
- If yesterday's brief has a "🚀 Release-Critical PRs" section, check each PR:
  - If PR is still open/unmerged today → include in today's Release-Critical section
  - If PR is merged → omit from today's release section
  - If deployment sequence exists (A blocks B), preserve the chain

For each archived brief, identify:
- Uncompleted tasks (checkboxes still marked `- [ ]`)
- Tasks that were in "Likely Carryovers" section
- High priority items that remain open
- **Release-Critical PRs from yesterday** (full read only)

### Step 3: Read Today's Raw Data

**IMPORTANT: Use `Bash ls` to discover sync files — do NOT use Glob (Glob cannot expand `~` and will silently return no results even with absolute paths on some systems).**

First, list the files using Bash:
```bash
ls $WORK_OS_BASE_DIR/raw/{TODAY-DATE}/
```

Then read each `sync-*.md` file found using the Read tool with its full absolute path:
```
$WORK_OS_BASE_DIR/raw/{TODAY-DATE}/sync-HHMM.md
```

**Structure:**
- Date folder: `raw/YYYY-MM-DD/`
- Sync files: `sync-HHMM.md` (24-hour time format)

For example, if today is 2026-01-23 and `ls` returns `sync-0943.md sync-1430.md`:
- Read `$WORK_OS_BASE_DIR/raw/2026-01-23/sync-0943.md`
- Read `$WORK_OS_BASE_DIR/raw/2026-01-23/sync-1430.md`

**IMPORTANT:**
- Read ALL `sync-*.md` files in today's date folder
- Do NOT read files from previous date folders (e.g., `raw/2026-01-22/`)
- Granola MOMs are now in `raw/YYYY-MM-DD/moms/` but are NOT read directly (already processed in sync files)

### Step 3.5: Read AI Session Log

This skill runs at the **start of the day**, so the primary data source is the **previous work day's** JSONL — today's file will be empty or nonexistent. Also check today's file for any prompts already logged (mid-day runs).

**Step 3.5a — Resolve previous work day date:**

```
If today is Monday → previous work day = last Friday (today - 3 days)
If today is Tuesday–Friday → previous work day = yesterday (today - 1 day)
If today is Saturday or Sunday → previous work day = last Friday
```

**Step 3.5b — Read previous work day JSONL:**

```bash
ls $WORK_OS_BASE_DIR/raw/{PREV-WORK-DATE}/ai-sessions.jsonl
```

If found, read and parse all lines. Label this data as **"Yesterday"** in the output.

**Step 3.5c — Read today's JSONL (optional):**

```bash
ls $WORK_OS_BASE_DIR/raw/{TODAY-DATE}/ai-sessions.jsonl
```

If found and has `event == "prompt"` records, include as **"Today (so far)"**. If empty or absent, skip silently.

**Step 3.5d — Compute from today's file (snapshot at time of generation):**

- **Total prompts** — count of records where `event == "prompt"`
- **Total sessions** — count of distinct `session_id` values
- **Providers** — unique providers used (claude, cursor, etc.)
- **Direction bar** — derive from total prompt count:
  - 🔴 High (> 15 prompts): `[████████░░]`
  - 🟡 Moderate (7–15 prompts): `[█████░░░░░]`
  - 🟢 Light (≤ 6 prompts): `[███░░░░░░░]`

If `ai-sessions.jsonl` does not exist, omit the Direction Effort lines from the AI Usage Statistics section.

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
   npx ccusage@latest daily --json
   ```

2. **Calculate Diff:**
   - Parse the new JSON output. Find the object in the `daily` array where `date` matches today's date.
   - Extract the current values as `END_...` variables (corresponding to the JSON keys used in Step 0).
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

1. Run `npx ccusage@latest daily --json` at the **start** of execution (before any tool calls)
2. Complete ALL work (file reads, brief generation, file writes)
3. Run `npx ccusage@latest daily --json` at the **end** of execution
4. Calculate the difference for today's date entry

### Important Notes

- **Focus on Total Cost** as the primary metric - this is most accurate and meaningful
- **Output tokens** may appear lower than expected because they count conversational responses, not necessarily content written to files via tools
- **Cache metrics** (Create/Read) will be high due to system prompts and skill instructions being reused
- Describe the **actual work performed** (files read/written) rather than trying to explain token-by-token breakdown
- If metrics seem inconsistent, acknowledge the discrepancy rather than guessing

### Output Format

```markdown
## 💰 Generation Cost

**Total Cost:** $X.XX

**Breakdown:**
- **Files Read:** N files (describe: raw data size, archives, etc.)
- **Files Written:** N files (describe: brief size, etc.)
- **Total Session Tokens:** ~XXX,XXX tokens processed
- **Cache Usage:** Heavy/Moderate/Light (brief description)

*Cost measured via `npx ccusage@latest daily --json` diff*
*Note: [Any discrepancies or observations about the metrics]*
```

---

## Learning / Improvements Section Guidelines

Extract insights from the raw data that indicate:
- Documentation gaps (e.g., "API contract discussions highlight need for stricter schema validation earlier")
- Process improvements (e.g., "Slack-based clarifications indicate documentation gaps in PROJECT flows")
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
