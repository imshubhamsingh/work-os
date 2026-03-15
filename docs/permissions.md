# Permissions Reference

All credentials, token scopes, and system permissions required to run Work-OS, consolidated in one place.

---

## Quick Reference

| Service | Token Type | Minimum Scopes / Access |
|---------|-----------|--------------------------|
| GitHub | Classic PAT or Fine-grained PAT | `repo`, `read:org`, `read:user` |
| Slack | User token (`xoxp-...`) | 12 user scopes (see below) |
| Jira | API token (Basic Auth) | Browse Projects + View Issues per project |
| Granola | None — local filesystem | Read access to `~/Library/Application Support/Granola/` |
| Coralogix | Logs Query API key | Read-only access to logs via DataPrime API |
| Google | OAuth2 (browser flow) | `calendar.readonly`, `tasks.readonly` |

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
| `files:read` | List and download canvas files (`files.list`, `files.info`, `url_private_download`) |
| `canvases:read` | Read canvas backing channel messages (`conversations.history` on the `C`-prefixed channel derived from the canvas file ID) |

> These are **User Token Scopes**, not Bot Token Scopes. Slack shows both sections on the OAuth & Permissions page — make sure you're adding to the correct one.

### What these permissions access

- Your DMs, group DMs, and monitored channel history
- @mentions and keyword matches via search
- Messages you sent (for follow-up tracking)
- User display name resolution
- Canvas files you are an editor of, or that are shared in monitored channels
- Canvas backing channel comments (inline canvas comments appear as messages in a channel with the same ID as the canvas file, with `F` replaced by `C`)
- Canvas content downloaded as HTML for mention detection and archiving

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

## Coralogix

**Token type:** Logs Query API key

Create at: Your Coralogix dashboard → **Data Flow → API Keys → Logs Query Key**

This is a **read-only** key scoped specifically to log queries. It is distinct from the Send-Your-Data key (ingestion) and the Alerts API key — do not use those.

| What it accesses | Details |
|------------------|---------|
| DataPrime query API | `POST https://api.eu1.coralogix.com/api/v1/dataprime/query` |
| Log records | ERROR-severity logs for your configured application names, production environment only |
| Log fields | Timestamp, severity, body, error message, service, trace/span IDs, environment |

> The API key is region-specific. If your Coralogix account is in a different region (e.g. US1, AP1), update the endpoint accordingly — but the key type and permissions are the same.

### What these permissions access

- Production ERROR logs for the applications you configure
- Log record metadata (timestamp, severity, logid for deduplication)
- No write access — the plugin only queries, never ingests

---

## Google

**Token type:** OAuth2 (browser-based flow)

Authenticate with:

```bash
work-os auth google
```

This covers both `google_calendar` and `google_tasks` — one auth, both plugins.

Google credentials (client ID + secret) are embedded at build time via `.cargo/config.toml`, not stored in `config.toml`. The per-user OAuth token is stored under `[plugins.google]` after authentication.

### OAuth2 scopes requested

| Scope | Plugin | Why |
|-------|--------|-----|
| `https://www.googleapis.com/auth/calendar.readonly` | Google Calendar | Read events from your primary calendar |
| `https://www.googleapis.com/auth/tasks.readonly` | Google Tasks | Read task lists and tasks |

Both scopes are read-only. Work-OS never creates, modifies, or deletes calendar events or tasks.

### What these permissions access

- Events on your primary Google Calendar within the sync date range
- All task lists in your Google Tasks account
- Incomplete tasks within each list (completed tasks are excluded)
- Event metadata: title, time, attendees, RSVP status, meeting link, description
- Task metadata: title, notes, due date, list name, subtask relationship

### Token storage

After authentication, the token is stored in `~/.work-os/config.toml` under `[plugins.google]`:

```toml
[plugins.google]
access_token = "ya29...."
refresh_token = "1//..."
expires_at = 1749999999
```

The access token is refreshed automatically. The refresh token persists until you revoke access in your [Google Account settings](https://myaccount.google.com/permissions).

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
