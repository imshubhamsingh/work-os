# My Workflow

This is how I actually use Work-OS day to day. It's less of a "here's a feature list" and more of a "here's the system I've built around it."

## The Obsidian vault structure

Everything lands in a single folder in my Obsidian vault:

```
00-work-os/
├── README.md                   ← start here: system overview and quick reference
├── raw/                        ← raw sync output, one folder per day
│   └── 2026-02-11/
│       ├── sync-0915.md        ← GitHub + Slack + Jira + Coralogix dump
│       ├── coralogix.jsonl     ← production error log (append-only, deduped)
│       └── moms/               ← Granola meeting notes, one folder per meeting
│           └── Meeting-Name/
│               ├── summary.md
│               └── transcript.md
├── archive/                    ← processed briefs, kept forever
│   ├── daily/
│   │   └── 2026-02-10.md
│   ├── weekly/
│   │   └── 2026-02-02_2026-02-07.md
│   └── monthly/
│       └── 2026-02.md
├── meetings/                   ← AI/hand-written meeting notes (separate from Granola)
├── follow-ups.md               ← persistent ledger of items waiting on others
└── today.md                    ← current day's brief (gets archived at end of day)
```

---

## Morning routine

**Step 1 — Sync**

```bash
work-os sync --mode since-last-run --markdown
```

This pulls GitHub PRs/reviews, Slack DMs/mentions/channels, Jira tickets, Granola meeting notes, and Coralogix production errors from since the last run. Dumps everything into `raw/{today}/sync-HHMM.md` and appends new error logs to `raw/{today}/coralogix.jsonl`.

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
| **Production Errors** | Coralogix error summary per application — recurring, one-offs, trend vs previous day, and any Slack threads where users reported the same issue |
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

## Meeting notes (Granola)

Granola records every meeting on my Mac and writes a summary + transcript into the local cache. Work-OS reads those and includes them in the raw sync output.

They show up in two places:
- As `moms/{meeting-name}/summary.md` and `transcript.md` under `raw/{date}/`
- Inline in `sync-HHMM.md` as `[GRANOLA]` items

The daily brief then pulls the relevant summaries into the **Context** section so I don't have to re-read transcripts.

---

## Production error monitoring

`work-os sync` queries Coralogix for ERROR-severity logs from configured applications and writes them to `raw/{today}/coralogix.jsonl` (append-only, deduped by log ID). The sync file also contains a per-application summary with recurring errors, one-offs, and trend arrows vs the previous day.

When `/work-os-today` generates the brief, it:

1. Takes the structured error summary already in the sync file verbatim
2. Reads `coralogix.jsonl` and scans today's Slack messages for any threads that mention the same service names, error bodies, or incident language
3. Synthesizes 3–5 pattern observations — which error classes are worsening, the most actionable root cause fix, any cascading failure chains
4. If Slack threads match a specific error, surfaces them under **Reported by Users in Slack** with both the Slack link and a direct Coralogix permalink side by side

The result in `today.md` looks like:

```
## 🚨 Production Errors · my-backend-service (142 errors ↑ from 98 prev)

### 🔁 Recurring — needs attention
| Count | Trend | Error | Link |
| 87x | ↑ 45→87 | Failed to process payment | [→](...) |

### ⚠️ One-off Concerns
- `Invalid request payload` (3x 🆕) — [→](...)

### 💡 Patterns
- Payment failures are worsening — upstream timeout cascade from auth service
- Guardian 401s suggest token expiry — check refresh interval in config

#### 📣 Reported by Users in Slack
- **[#support-channel](slack_url)** — "@user payment failing for MMP 1234"
  → `Failed to process payment` · 87x · [Coralogix →](coralogix_url)
```

This section appears every day automatically as long as Coralogix is configured and the sync ran.

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

The brief gets archived to `archive/daily/{date}.md` automatically when the next day's brief is generated.

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

## Monthly summary

At the start of each month, I run:

```
/work-os-monthly
```

This reads all weekly summaries for the prior month and generates `archive/monthly/{YYYY-MM}.md`. Unlike the weekly which asks "was this a good week?", the monthly operates at a higher resolution — it asks what kind of engineer I'm becoming and what systemic issues need structural fixes.

It produces:

| Section | What's in it |
|---------|-------------|
| **Executive Summary** | High-level narrative a manager can read standalone — sentiment, delivery health, key win, systemic risk |
| **Month at a Glance** | PRs, reviews, after-hours nights, AI %, carryover peak — with trend vs prior month |
| **Time Allocation** | Where time actually went (execution / reviews / coordination / firefighting) vs personal targets, broken down per week |
| **Features Shipped** | Shipped work grouped by feature area, with the full lifecycle arc from start to production |
| **Code Output** | Monthly LOC totals with AI vs human split, weekly breakdown |
| **Review Throughput** | Review count, average per week, and whether the load was sustainable |
| **AI Adoption Trajectory** | Weekly AI % trend plus how AI was used — boilerplate, architecture exploration, debugging, spec drafting |
| **Oscillation Summary** | Which weeks were sustainable vs grinding, what caused the load peaks, stress attribution |
| **Work Type Distribution** | Balance between hands-on delivery, leverage work (tooling, standards, cross-team unblocking), and reactive work |
| **Ceiling Raise Index** | Things done this month that permanently make the team faster next month |
| **Delegation Score** | What was delegated vs what could have been, with a ratio and pattern observation |
| **Systemic Patterns** | Issues that appeared in 3+ weeks — structural, not situational |
| **Drag Log** | Recurring friction, whether a fix was attempted, and what's still unaddressed going into next month |
| **Kill List** | What was deliberately stopped doing this month — productivity increases from subtraction too |
| **Next Month Commitments** | Max 3 structural changes derived from patterns — concrete, observable, within your control |
| **Confidence Delta** | End-of-month self-assessment across workload control, strategic clarity, and energy headroom |

The monthly is for stepping back and asking what's systemic. The weekly is what you'd show a manager. The daily is just for you.

---

## What this actually saves

Before this: 30-45 minutes every morning piecing together context from Slack, GitHub, and Jira tabs.

After: ~5 minutes to run sync and generate the brief, then I'm straight into the actual work.

The bigger win is the end-of-day reflection and follow-ups ledger. Without a system, things that are "blocked on someone else" just disappear into the void. Now they surface every morning until they resolve.
