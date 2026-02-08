---
description: Maintain and reconcile long-lived follow-ups from daily and weekly work context
model: opus
---

# Work OS — Follow-ups

You are managing **stateful follow-ups** for the Work OS.

Follow-ups represent **waiting states** or **external dependencies** that must not get buried in daily or weekly summaries.

This command is the **single source of truth** for creating, updating, and resolving follow-ups.

---

## Core Principles (NON-NEGOTIABLE)

- Follow-ups are **stateful**, not snapshots
- This command is the **only command allowed to mutate follow-ups**
- Resolution is **monotonic** (once resolved, never resurrected)
- Prefer **doing less** over doing more
- If unsure, **do nothing**

---

## Canonical File

All follow-ups live in a single file:

```
$WORK_OS_BASE_DIR/follow-ups.md
```

**Environment Variables:**
- `$WORK_OS_BASE_DIR`: Base directory for work-os data
  - Example: `~/Projects/obsidian/work/00-work-os`

This file is authoritative.

---

## File Structure (MANDATORY)

The follow-ups file MUST contain exactly two top-level sections:

```markdown
# Follow-ups

## Active
## Resolved
```

- **Active** → all currently waiting follow-ups
- **Resolved** → completed follow-ups kept for reference and audit

No other top-level sections are allowed.

---

## Follow-up Lifecycle

A follow-up can be in one of two states:

- **Active** — still waiting on an external dependency or future action
- **Resolved** — completed and closed

Once resolved, a follow-up must never be reactivated.

If a similar situation arises later, create a **new follow-up**.

---

## Follow-up Entry Structure (STRICT)

### Active Follow-up

```markdown
- [ ] **<Short title>**
  - Waiting on: <person / system / condition>
  - Since: YYYY-MM-DD
  - Last checked: YYYY-MM-DD
  - Context: <1-line explanation>
  - Source: [[YYYY-MM-DD#Heading|source]]
  - Link: <optional: PR / Slack / Jira / external URL>
```

### Resolved Follow-up

```markdown
- [x] **<Short title>**
  - Resolved: YYYY-MM-DD
  - Resolution: <why it was resolved>
```

---

## Command Responsibilities

This command performs **four actions only**:

1. Read existing follow-ups
2. Detect new follow-ups
3. Resolve completed follow-ups
4. Persist updated follow-ups

It must NOT:
- Write to `today.md`
- Modify archived daily or weekly files
- Reinterpret historical content
- Invent follow-ups retroactively

---

## Input Sources (READ-ONLY)

The command may read:

- `today.md`
- Archived daily briefs (last 7 days)
- Latest weekly summary (if present)

All input sources are **read-only**.

---

## Follow-up Creation Rules

Create a new follow-up ONLY when all conditions are true:

- There is an explicit **waiting state**
- Ownership is **external** or delayed
- The item is expected to require future action

Common creation signals:
- "waiting on"
- "pending review"
- "blocked by"
- "follow up after"
- "check once X is done"

Rules:
- Do NOT create duplicates
- Do NOT infer urgency
- Do NOT create speculative follow-ups
- If a similar resolved follow-up exists, create a **new entry**, not reuse it

---

## Follow-up Resolution Rules

Use a **hybrid resolution model**.

### 1. Automatic Resolution (HIGH CONFIDENCE)

Resolve automatically if ANY of the following are detected:

- A referenced PR is **merged or closed**
- Notes explicitly state "done", "shipped", "merged", or "resolved"
- The blocking condition no longer exists

---

### 2. Soft Resolution (DEFAULT)

Resolve if:
- The waiting condition is implicitly cleared
- You are no longer waiting (e.g. review completed, response received)

If ambiguity exists, prefer **resolution over persistence**.

---

### 3. Manual Resolution (ABSOLUTE)

If a follow-up is manually marked `[x]`, it is final.

Never question, override, or reopen a manually resolved follow-up.

---

## Resolution Guarantees

- Once resolved, a follow-up must never reappear
- Resolved follow-ups must remain visible under `## Resolved`
- Resolved items must never be re-evaluated or moved back to `## Active`
- Resolution must always record:
  - resolution date
  - reason

---

## Persistence Rules

When updating `follow-ups.md`:

- All open follow-ups MUST live under `## Active`
- All completed follow-ups MUST be moved to `## Resolved`
- Do NOT delete resolved follow-ups
- Preserve original `Since` dates for active items
- Update `Last checked` for active items
- Maintain chronological order:
  - **Active:** oldest first
  - **Resolved:** most recently resolved first

---

## Output Rules

Write the updated follow-ups to:

```
$WORK_OS_BASE_DIR/follow-ups.md
```

No other output is required.

Do NOT print explanations.
Do NOT summarize changes.
Do NOT ask questions.

---

## Invocation

```
/work-os-follow-ups
```

---

## Quality Bar

- Follow-ups should feel **quiet and trustworthy**
- No duplicate entries
- No resurrected follow-ups
- Minimal churn between runs
- If uncertain, prefer inaction
