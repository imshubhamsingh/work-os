# Slack Plugin

Fetches DMs, group DMs, channel activity, mentions, keyword matches, your own sent messages, and Slack Canvases.

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
| `files:read` | Read canvas files and their metadata |
| `canvases:read` | Access canvas content and backing channel messages |

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
        CV["Canvases"]
    end

    subgraph Output
        Msg["Message items\n(MessageType::Message)"]
        CanvasMsg["Canvas items\n(MessageType::Canvas)"]
        CanvasFile["Canvas .md files\n(raw/DATE/canvases/)"]
    end

    Slack --> DM & GDM & CH & KW & Mentions & Mine & CV
    DM & GDM & CH & KW & Mentions & Mine --> Msg
    CV --> CanvasMsg
    CV --> CanvasFile
```

| Source | Description |
|--------|-------------|
| Direct Messages | 1-on-1 DMs with other users |
| Group DMs | Multi-person DMs (MPIM) |
| Channel Messages | Activity in monitored channels |
| Keyword Matches | Messages containing configured keywords |
| Mentions | Messages where you are `@mentioned` |
| Your Messages | Messages you sent (for follow-up tracking) |
| Canvases | Slack Canvas documents (see below) |

## Canvas Fetching

Canvases are processed in two buckets based on your involvement.

### Directly Involved (editor or monitored channel)

You are considered directly involved if:
- You appear in the canvas `editors` list, or
- The canvas is shared in one of your configured `channels`

For these canvases:
1. The backing channel (`F{id}` → `C{id}`) is fetched for comments within the date range
2. The canvas HTML is downloaded and checked for `<@YOU>` mentions
3. Share threads from other channels are scanned for mention replies
4. A canvas `.md` file is always written to `raw/DATE/canvases/TITLE/`
5. A sync `Message` is always emitted with channel comments attached

### Observer (everything else in date range)

For canvases you are not directly involved with:
1. The backing channel is fetched for comments within the date range
2. If `<@YOU>` is found anywhere in those messages, the canvas is included
3. A canvas `.md` file is written and a `[Mentioned]` sync `Message` is emitted
4. If no mention is found, the canvas is silently skipped

### Canvas backing channel

Every Slack canvas file has a backing conversation channel where inline comments appear as messages. The channel ID is derived directly from the file ID by replacing the leading `F` with `C` (e.g. `F0A9XPX5EEA` → `C0A9XPX5EEA`).

### Canvas output files

Canvas `.md` files are written to:
```
raw/YYYY-MM-DD/canvases/CANVAS-TITLE/YYYY-MM-DD-HHmm.md
```

Each file contains:
- Canvas title, Slack URL, and last updated timestamp
- Comments where you are @mentioned (from share threads)
- Full rendered canvas content (HTML converted to Markdown)

## Canvas Flow

```mermaid
sequenceDiagram
    participant Plugin as canvas.rs
    participant API as Slack API
    participant Writer as CanvasWriter

    Plugin->>API: files.list?types=canvas (paginated)
    API-->>Plugin: SlackCanvas[]

    Note over Plugin: Split into directly_involved / others

    loop Directly involved canvases
        Plugin->>API: conversations.history (backing channel F→C)
        API-->>Plugin: channel comments
        Plugin->>API: files.info (share threads)
        API-->>Plugin: share metadata
        Plugin->>API: conversations.replies (per share with replies)
        API-->>Plugin: mention replies
        Plugin->>API: url_private_download (canvas HTML)
        API-->>Plugin: raw canvas content
        Plugin->>Writer: write canvas .md file (always)
        Plugin-->>Plugin: emit Canvas Message (always)
    end

    loop Other canvases
        Plugin->>API: conversations.history (backing channel F→C)
        API-->>Plugin: channel comments
        Note over Plugin: Skip if no @mention found
        Plugin->>API: url_private_download (canvas HTML)
        API-->>Plugin: raw canvas content
        Plugin->>Writer: write canvas .md file
        Plugin-->>Plugin: emit Canvas Message [Mentioned]
    end
```

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
