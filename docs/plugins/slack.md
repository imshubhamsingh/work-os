# Slack Plugin

Fetches DMs, group DMs, channel activity, mentions, keyword matches, and your own sent messages from Slack.

## Setup

```bash
work-os config set slack token xoxp-YOUR-USER-TOKEN

# Optional: keywords that signal action items
work-os config set slack keywords "please review,action required,LGTM"

# Optional: channels to monitor (leave empty for all)
work-os config set slack channels general,engineering,releases
```

**Token type required:** User token (`xoxp-...`), not a bot token.
Create at: https://api.slack.com/apps → OAuth & Permissions → User Token Scopes

## Permissions

| Scope | Why it's needed |
|-------|----------------|
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

> These are **User Token Scopes**, not Bot Token Scopes. Make sure you're adding them in the right section when configuring your Slack app.

## What It Fetches

```mermaid
flowchart TD
    Slack["Slack API"]

    subgraph Sources
        DM["Direct Messages"]
        GDM["Group DMs / MPIM"]
        CH["Channel Messages"]
        KW["Keyword Matches"]
        Mentions["Mentions & @you"]
        Mine["Your Sent Messages"]
    end

    subgraph Output
        Msg["Message items\n(MessageType::Message)"]
    end

    Slack --> DM & GDM & CH & KW & Mentions & Mine
    DM & GDM & CH & KW & Mentions & Mine --> Msg
```

| Source | Description |
|--------|-------------|
| Direct Messages | 1-on-1 DMs with other users |
| Group DMs | Multi-person DMs (MPIM) |
| Channel Messages | Activity in monitored channels |
| Keyword Matches | Messages containing configured keywords |
| Mentions | Messages where you are `@mentioned` |
| Your Messages | Messages you sent (for follow-up tracking) |

## Message Flow

```mermaid
sequenceDiagram
    participant Plugin as Slack Plugin
    participant API as Slack API
    participant Builder as build_message()

    Plugin->>API: conversations.list (im, mpim, public_channel)
    API-->>Plugin: channel list

    loop For each channel
        Plugin->>API: conversations.history
        API-->>Plugin: SlackMessage[]
        Plugin->>API: conversations.replies (for threads)
        API-->>Plugin: thread replies
        Plugin->>Builder: title + description + timestamp
        Builder-->>Plugin: Message
    end

    Plugin->>API: search.messages (keywords + mentions)
    API-->>Plugin: search results → Message[]
```

## Configuration Reference

| Key | Required | Description |
|-----|----------|-------------|
| `token` | ✅ | Slack user token (`xoxp-...`) |
| `keywords` | — | Comma-separated words to search for |
| `channels` | — | Channel names to monitor (empty = all) |

## CLI Usage

```bash
# Sync Slack only
work-os sync --plugins slack

# Combine with GitHub
work-os sync --plugins slack,github
```
