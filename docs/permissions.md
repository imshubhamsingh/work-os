# Permissions Reference

All credentials, token scopes, and system permissions required to run Work-OS, consolidated in one place.

---

## Quick Reference

| Service | Token Type | Minimum Scopes / Access |
|---------|-----------|--------------------------|
| GitHub | Classic PAT or Fine-grained PAT | `repo`, `read:org`, `read:user` |
| Slack | User token (`xoxp-...`) | 10 user scopes (see below) |
| Jira | API token (Basic Auth) | Browse Projects + View Issues per project |
| Granola | None — local filesystem | Read access to `~/Library/Application Support/Granola/` |

---

## GitHub

**Token type:** Personal Access Token (Classic or Fine-grained)

Create at: <https://github.com/settings/tokens>

### Classic PAT scopes

| Scope | Why |
|-------|-----|
| `repo` | Read PRs, commits, and issue comments from your repos |
| `read:org` | Read organisation membership — required to fetch PRs from org repos |
| `read:user` | Read your user profile — used to identify PRs you authored |

> If you only work in personal repos (not an org), `read:org` can be omitted.

### Fine-grained PAT (PAT v2) — equivalent permissions

| Permission | Level |
|-----------|-------|
| Pull requests | Read |
| Contents | Read |
| Metadata | Read (required by default) |
| Members (organisation) | Read |

### What these permissions access

- PRs where you are author, reviewer, or assignee
- Commits within those PRs (for AI usage scoring)
- Review comments and issue comments
- Organisation membership (to resolve org repo access)

---

## Slack

**Token type:** User token (`xoxp-...`) — **not** a bot token

Create at: <https://api.slack.com/apps> → Your App → OAuth & Permissions → **User Token Scopes**

| Scope | Why |
|-------|-----|
| `channels:history` | Read messages from public channels you're a member of |
| `channels:read` | List public channels |
| `groups:history` | Read messages from private channels you're a member of |
| `groups:read` | List private channels you're a member of |
| `im:history` | Read your 1-on-1 DMs |
| `im:read` | List your DMs |
| `mpim:history` | Read group DMs |
| `mpim:read` | List group DMs |
| `search:read` | Search messages for keywords and @mentions |
| `users:read` | Resolve user IDs to display names |

> These are **User Token Scopes**, not Bot Token Scopes. Slack shows both sections on the OAuth & Permissions page — make sure you're adding to the correct one.

### What these permissions access

- Your DMs, group DMs, and monitored channel history
- @mentions and keyword matches via search
- Messages you sent (for follow-up tracking)
- User display name resolution

---

## Jira

**Token type:** Atlassian API token (used as Basic Auth password)

Create at: <https://id.atlassian.com/manage-profile/security/api-tokens>

Jira API tokens don't have granular scopes — access is controlled by your **Jira project memberships**.

### Project-level permissions required (per project you query)

| Permission | Why |
|-----------|-----|
| Browse Projects | See the project and its issues at all |
| View Issues | Read issue fields: summary, description, status, assignee, reporter |

> If a JQL filter returns no results and you expect it to, the most likely cause is missing Browse Projects access on that project.

### What these permissions access

- Issues matching your configured JQL filters
- Issue fields: summary, description, status, priority, assignee, reporter, created/updated dates
- No write access is used or needed

---

## Granola

**Token type:** None — reads directly from the local filesystem

No credentials, no OAuth, no API keys.

| Resource | Path |
|----------|------|
| Meeting notes cache | `~/Library/Application Support/Granola/` |

This path is inside your own user Library directory and requires no special permissions under normal circumstances.

**If you get empty results from Granola,** check:
- Granola is installed and has recorded at least one meeting
- Your terminal emulator has **Full Disk Access** — System Settings → Privacy & Security → Full Disk Access

---

## Storing Credentials Securely

Work-OS stores credentials in `~/.config/work-os/config.toml`. This file is plain text — protect it:

```bash
# Restrict to owner read/write only
chmod 600 ~/.config/work-os/config.toml
```

The config file contains live tokens — do not commit it to version control. If you're checking your dotfiles into git, add `.config/work-os/` to your `.gitignore`.

---

## Verifying All Connections

```bash
# Test all configured plugins at once
work-os auth

# Test a specific plugin
work-os auth --plugin github
work-os auth --plugin slack
```

This runs a minimal read operation against each configured service and reports success or the specific error (wrong scope, expired token, missing project access, etc.).
