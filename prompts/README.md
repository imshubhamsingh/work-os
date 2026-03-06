# Work-OS Claude Command Templates

Generic templates for Work-OS daily, weekly, monthly, and follow-up commands.

## Templates Included

### 1. `work-os-today.md`
Generates daily work briefs from synced work-os raw Markdown data.

**What it does:**
- Reads raw Slack, GitHub, and Jira data
- Generates prioritized daily brief
- Tracks AI code usage statistics
- Manages carryovers and follow-ups

### 2. `work-os-weekly.md`
Generates weekly summaries from archived daily briefs.

**What it does:**
- Synthesizes 7 days of daily briefs
- Creates manager-readable lead summary
- Aggregates AI usage statistics with daily trend comparison
- Identifies wins, misses, and learnings

### 3. `work-os-monthly.md`
Generates monthly summaries from weekly summaries with career-level signal and systemic pattern analysis.

**What it does:**
- Synthesizes 4–5 weeks of weekly summaries into a high-signal monthly view
- Generates an executive-readable summary (VP/Director level)
- Tracks time allocation reality vs targets (execution, reviews, coordination, firefighting)
- Derives role load split (EM vs Staff Engineer vs IC)
- Produces AI adoption trajectory and leverage quality classification
- Identifies systemic patterns recurring across 3+ weeks
- Scores delegation health and ceiling-raise contributions
- Outputs promotion readiness signals and next-month structural commitments

### 4. `work-os-follow-ups.md`
Maintains stateful follow-ups across daily and weekly contexts.

**What it does:**
- Tracks items waiting on external dependencies
- Auto-resolves completed follow-ups
- Prevents follow-ups from getting lost
- Single source of truth for waiting states

---

## File Structure

Work-OS uses a date-based folder structure for organizing data:

```
$WORK_OS_BASE_DIR/
├── raw/
│   ├── 2026-02-08/
│   │   ├── sync-0943.md              # Sync at 09:43 AM
│   │   ├── sync-1430.md              # Sync at 14:30 PM
│   │   └── moms/                     # Granola meeting notes (if configured)
│   │       ├── meeting-name-1/
│   │       │   ├── transcript.md
│   │       │   └── summary.md
│   │       └── meeting-name-2/
│   │           ├── transcript.md
│   │           └── summary.md
│   ├── 2026-02-07/
│   │   ├── sync-0930.md
│   │   └── moms/
│   └── ...
├── today.md
├── archive/
│   ├── 2026-02-07.md
│   └── weekly/
│       └── 2026-02-01_2026-02-07.md
└── follow-ups.md
```

**Key Points:**
- Each day has its own folder: `raw/YYYY-MM-DD/`
- Sync files use 24-hour format: `sync-HHMM.md`
- Multiple syncs per day are supported
- Granola MOMs are organized under `moms/` within the date folder
- All raw data for a specific date is self-contained and easy to archive/delete

---

## Environment Variables

These templates use environment variables for portability. Configure based on your setup:

### Required Variables

- **`$WORK_OS_BASE_DIR`**: Base directory for work-os data
  - Example: `~/Projects/obsidian/work/00-work-os`
  - Used in all templates

### Optional Variables

- **`$GITHUB_USERNAME`**: Your GitHub username
  - Example: `imshubhamsingh`
  - Used in `work-os-today.md` for PR review detection

- **`$ACK_REACTION`**: Acknowledgment reaction emoji for Must-Do items
  - Example: `:ack:`
  - Used in `work-os-today.md`

- **`$ENABLE_COST_TRACKING`**: Enable token usage/cost tracking
  - Values: `true` | `false` (default: `false`)
  - Used in `work-os-today.md`, `work-os-weekly.md`

- **`$COST_TRACKING_CMD`**: Command to track token usage
  - Example: `npx ccusage@latest daily --json`
  - Used when `$ENABLE_COST_TRACKING` is `true`

### Example Configuration

**Full setup (like Shubham's):**
```bash
export WORK_OS_BASE_DIR="$HOME/Projects/obsidian/work/00-work-os"
export GITHUB_USERNAME="imshubhamsingh"
export ACK_REACTION=":ack:"
export ENABLE_COST_TRACKING="true"
export COST_TRACKING_CMD="npx ccusage@latest daily --json"
```

**Minimal setup:**
```bash
export WORK_OS_BASE_DIR="$HOME/work-os"
```

---

## Customization

To use these templates for your own setup:

### 1. Configure Environment Variables

Set the environment variables above in your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) or `.env` file.

### 2. Adjust Integration Sources

**Default integrations:**
- Templates assume Slack, GitHub, and Jira
- Remove or add integrations based on your work-os setup
- Update plugin references as needed

### 3. Adjust Classification Rules

Edit the classification rules in `work-os-today.md` to match your workflow:
- Must Do criteria
- Review thresholds
- Context grouping preferences

### 4. Customize Output Structure

Modify section headers and content to match your needs:
- Add/remove sections
- Change emoji indicators
- Adjust formatting preferences

### 5. Configure AI Stats (Optional)

If you're not using AI code tracking:
- Remove the `## 🤖 AI Usage Statistics` section
- Skip AI stats aggregation in weekly summary

---

## Installation

1. Copy templates to your `.claude/commands/` directory:
   ```bash
   cp .claude/templates/*.md ~/.claude/commands/
   ```

2. Customize as needed (see Customization above)

3. Invoke commands:
   ```
   /work-os-today
   /work-os-weekly
   /work-os-monthly
   /work-os-follow-ups
   ```

---

## Template Philosophy

These templates are designed to be:
- **Generic**: No company or personal references
- **Customizable**: Easy to adapt to your workflow
- **Opinionated**: Strong defaults, but flexible
- **Maintainable**: Clear structure, well-documented

---

## Contributing

If you improve these templates:
1. Keep them generic (no personal/company info)
2. Document customization points
3. Maintain backward compatibility when possible

---

## License

MIT - Use freely, customize as needed
