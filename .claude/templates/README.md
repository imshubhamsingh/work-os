# Work-OS Claude Command Templates

Generic templates for Work-OS daily, weekly, and follow-up commands.

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

### 3. `work-os-follow-ups.md`
Maintains stateful follow-ups across daily and weekly contexts.

**What it does:**
- Tracks items waiting on external dependencies
- Auto-resolves completed follow-ups
- Prevents follow-ups from getting lost
- Single source of truth for waiting states

---

## Customization

To use these templates for your own setup:

### 1. Replace Placeholders

**GitHub Username:**
- Find: `{YOUR_GITHUB_USERNAME}`
- Replace with: your actual GitHub username (e.g., `octocat`)

**File Paths:**
- Update paths to match your Obsidian vault structure
- Default: `~/Projects/obsidian/work/00-work-os/`
- Customize to your vault location

**Integration Sources:**
- Templates assume Slack, GitHub, and Jira
- Remove or add integrations based on your work-os setup

### 2. Adjust Classification Rules

Edit the classification rules in `work-os-today.md` to match your workflow:
- Must Do criteria
- Review thresholds
- Context grouping preferences

### 3. Customize Output Structure

Modify section headers and content to match your needs:
- Add/remove sections
- Change emoji indicators
- Adjust formatting preferences

### 4. Configure AI Stats (Optional)

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
