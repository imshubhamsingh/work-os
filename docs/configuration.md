# Configuration

Work-OS stores its config at `~/.config/work-os/config.toml`.

Run `work-os config init` to set it up interactively, or edit the file directly.

## Full Example

```toml
[output]
base_path = "/path/to/obsidian/vault/work-os"
markdown_path = "raw"

[plugins.github]
enabled = true
username = "your-github-username"
token = "ghp_..."
include_orgs = ["myorg"]
include_repos = [
    "myorg/frontend",
    "myorg/backend",
]
bots = [
    "dependabot[bot]",
    "renovate[bot]",
]

[plugins.slack]
enabled = true
token = "xoxp-..."
keywords = [
    "@my-team",
    "@oncall",
]
channels = [
    "C0A7MQP33K7",   # channel IDs, not names
    "C092RJAL9EW",
]
max_messages_per_channel = 50

[plugins.jira]
enabled = true
domain = "company.atlassian.net"
email = "you@company.com"
token = "..."

[[plugins.jira.filters]]
name = "My Active Tickets"
jql = "project = EM AND assignee = currentUser() AND statusCategory != Done"
priority = "medium"

[[plugins.jira.filters]]
name = "Needs Review"
jql = "project = HUBPE AND assignee = currentUser() AND statusCategory != Done"
priority = "medium"

[plugins.granola]
enabled = true
```

---

## `[output]`

Controls where generated markdown files are written.

| Key | Required | Description |
|-----|----------|-------------|
| `base_path` | ✅ | Root directory for output files (e.g. your Obsidian vault) |
| `markdown_path` | ✅ | Subdirectory under `base_path` for raw sync output |

Files are written to `{base_path}/{markdown_path}/{date}/sync-{HHMM}.md`.

---

## `[plugins.github]`

| Key | Required | Description |
|-----|----------|-------------|
| `enabled` | ✅ | `true` / `false` |
| `token` | ✅ | Personal Access Token — [create one](https://github.com/settings/tokens) with `repo`, `read:org`, `read:user` |
| `username` | ✅ | Your GitHub username |
| `include_orgs` | — | Only fetch PRs from these orgs. Leave empty for all. |
| `include_repos` | — | Only fetch PRs from these repos (`owner/repo`). Takes precedence over `include_orgs`. |
| `bots` | — | Bot account names to exclude from review/comment data |

---

## `[plugins.slack]`

| Key | Required | Description |
|-----|----------|-------------|
| `enabled` | ✅ | `true` / `false` |
| `token` | ✅ | User token (`xoxp-...`) — [create one](https://api.slack.com/apps) |
| `keywords` | — | Messages containing these strings are surfaced as action items |
| `channels` | — | Channel IDs to monitor (use IDs, not names). Leave empty for all accessible channels. |
| `max_messages_per_channel` | — | Cap on messages fetched per channel (default: 50) |

> **Finding a channel ID:** In Slack, open the channel → click the channel name at the top → scroll to the bottom of the popup — the ID starts with `C`.

---

## `[plugins.jira]`

| Key | Required | Description |
|-----|----------|-------------|
| `enabled` | ✅ | `true` / `false` |
| `domain` | ✅ | Atlassian domain, e.g. `company.atlassian.net` |
| `email` | ✅ | Your Atlassian account email |
| `token` | ✅ | API token — [create one](https://id.atlassian.com/manage-profile/security/api-tokens) |

### `[[plugins.jira.filters]]`

Each filter defines a JQL query to run. You can have as many as you need.

| Key | Required | Description |
|-----|----------|-------------|
| `name` | ✅ | Display name shown in output |
| `jql` | ✅ | JQL query string |
| `priority` | — | Default priority for matching issues: `critical`, `high`, `medium`, `low` |

**Common JQL patterns:**

```
# Your active tickets in a project
project = EM AND assignee = currentUser() AND statusCategory != Done

# In Review across all projects
assignee = currentUser() AND status = "In Review"

# Blocked tickets
assignee = currentUser() AND labels = blocked
```

---

## `[plugins.granola]`

| Key | Required | Description |
|-----|----------|-------------|
| `enabled` | ✅ | `true` / `false` |

No other configuration needed — Granola reads from a fixed local cache path.

---

## CLI Shortcuts

```bash
# View current config
work-os config show

# View config for one plugin
work-os config show github

# Set a value
work-os config set github username imshubhamsingh

# Re-run interactive setup for a plugin
work-os config init github

# Test all connections
work-os auth
```
