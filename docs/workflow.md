# My Workflow

This is how I actually use Work-OS day to day. It's less of a "here's a feature list" and more of a "here's the system I've built around it."

## The Obsidian vault structure

Everything lands in a single folder in my Obsidian vault:

```
00-work-os/
├── README.md                   ← start here: system overview and quick reference
├── raw/                        ← raw sync output, one folder per day
│   └── 2026-02-11/
│       └── sync-0915.md        ← GitHub + Slack + Jira dump
├── archive/                    ← processed daily briefs, kept forever
│   ├── 2026-02-10.md
│   └── weekly/
│       └── 2026-02-02_2026-02-07.md
├── meetings/                   ← AI/hand-written meeting notes
├── follow-ups.md               ← persistent ledger of items waiting on others
└── today.md                    ← current day's brief (gets archived at end of day)
```

---

## Morning routine

**Step 1 — Sync**

```bash
work-os sync --mode since-last-run --markdown
```

This pulls GitHub PRs/reviews, Slack DMs/mentions/channels, and Jira tickets from since the last run. Dumps everything into `raw/{today}/sync-HHMM.md`.

**Step 2 — Generate the daily brief**

In Claude Code, inside the vault:

```
/work-os-today
```

This reads the raw sync file, cross-references yesterday's archive and the follow-ups ledger, and writes a structured `today.md`.

That's it. Usually done in under 5 minutes.

---

## What the daily brief looks like

`today.md` has a fixed structure that Claude generates from the raw data:

| Section | What's in it |
|---------|-------------|
| **Top 3 Outcomes** | The 3 things I'm committing to finish today |
| **Must Do** | Grouped action items with Slack/PR/Jira links |
| **Reviews / Approvals** | Pending PRs split into "needs review" and "reviewed, waiting on author" with age |
| **Likely Carryovers** | Items that probably won't close today, with how long they've been open |
| **Follow-ups & Waiting** | Pulled from `follow-ups.md`, things blocked on someone else |
| **Risks / Blockers** | Anything that might blow up the day |
| **Context** | Meeting summaries and important Slack threads, for awareness only |
| **AI Usage Stats** | LOC breakdown from GitHub commits, AI vs human |
| **Learning / Improvements** | One or two things worth remembering from the day's raw data |
| **End of Day Reflection** | Written by me, not AI, at the end of the day |
| **Generation Cost** | Token cost of the brief, tracked out of curiosity |

The "End of Day Reflection" is the only section I ~~write manually~~ use wispr flow. Everything else is generated from data.

---

## Follow-ups ledger

`follow-ups.md` is a single file that tracks everything I'm waiting on someone else for:

```markdown
## Active

- [ ] **Testing View: backend data model**
  - Waiting on: @abc.colleague
  - Since: 2026-02-07
  - Last checked: 2026-02-10
  - Context: finalize affected APIs and migration plan by Monday

## Resolved

- [x] **XYZ integration release**
  - Resolved: 2026-01-27
  - Resolution: All PRs merged and deployed
```

The `/work-os-today` skill reads this file and surfaces any overdue items in the **Follow-ups & Waiting** section of the daily brief. When something resolves, it moves to `## Resolved` with a note.

The ledger stays small because resolved items just pile up at the bottom rather than getting deleted.

---

## End of day

At the end of the day I:
1. Write a few lines of reflection in the `## End of Day Reflection` section (voice-to-text or typed, rough is fine)
2. Update any follow-ups that resolved or got new info
3. Run `/work-os-today` one more time (or let the next morning's run archive it)

The brief gets archived to `archive/{date}.md` automatically when the next day's brief is generated.

---

## Weekly summary

On Monday mornings, I run:

```
/work-os-weekly
```

This reads the week's archived daily briefs and generates a `archive/weekly/{start}_{end}.md` with:
- A lead-level summary (sentiment, delivery confidence, key wins/misses)
- Shipped work and slippage
- Trends across the week (review queue depth, carryover patterns, after-hours incidents)
- Something worth carrying forward

The weekly is the thing I'd show a manager. The daily is just for me.

---

## What this actually saves

Before this: 30-45 minutes every morning piecing together context from Slack, GitHub, and Jira tabs.

After: ~5 minutes to run sync and generate the brief, then I'm straight into the actual work.

The bigger win is the end-of-day reflection and follow-ups ledger. Without a system, things that are "blocked on someone else" just disappear into the void. Now they surface every morning until they resolve.
