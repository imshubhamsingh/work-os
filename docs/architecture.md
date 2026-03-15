# Architecture

Work-OS is built around a plugin registry that collects `Message` items from multiple sources, then feeds them into output generators.

## System Overview

```mermaid
flowchart TD
    CLI["CLI (clap)"]
    Registry["PluginRegistry"]
    GitHub["🐙 GitHub Plugin"]
    Slack["💬 Slack Plugin"]
    Jira["🎫 Jira Plugin"]
    Granola["🍥 Granola Plugin"]
    Coralogix["🚨 Coralogix Plugin"]
    GoogleAuth["🔑 Google Plugin\n(auth only)"]
    GCal["📅 Google Calendar Plugin"]
    GTasks["📋 Google Tasks Plugin"]
    Core["core::Message"]
    TermOutput["Terminal Output"]
    JSONOutput["JSON Output"]
    MDOutput["Markdown Generator"]

    CLI -->|sync / stats| Registry
    Registry --> GitHub
    Registry --> Slack
    Registry --> Jira
    Registry --> Granola
    Registry --> Coralogix
    Registry --> GoogleAuth
    Registry --> GCal
    Registry --> GTasks
    GoogleAuth -.->|shared OAuth token| GCal
    GoogleAuth -.->|shared OAuth token| GTasks
    GitHub -->|Vec<Message>| Core
    Slack -->|Vec<Message>| Core
    Jira -->|Vec<Message>| Core
    Granola -->|Vec<Message>| Core
    Coralogix -->|Vec<Message> + JSONL| Core
    GCal -->|Vec<Message>| Core
    GTasks -->|Vec<Message>| Core
    Core --> TermOutput
    Core --> JSONOutput
    Core --> MDOutput
```

## Core Concepts

### `Message`

Everything in Work-OS becomes a `Message`. It is the single unified model passed between plugins and output generators:

```
Message {
  id          — unique key: "source:type:id"
  source      — "github" | "slack" | "jira" | "granola" | "google_calendar" | "google_tasks"
  message_type — PullRequest | Issue | Review | Message | Ticket | Statistics | MOM | Canvas | Coralogix | CalendarEvent | GoogleTask
  title
  description
  url
  priority    — Critical | High | Medium | Low | Unknown
  status      — Open | InProgress | Blocked | Done
  created_at / updated_at
  people      — Vec<Person> with roles (Author, Assignee, Reviewer, Mentioned)
  metadata    — MessageMetadata::GitHub(...) | None
}
```

### Plugin Trait

Every plugin implements:

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn is_configured(&self) -> bool;
    fn config_schema(&self) -> Vec<ConfigField>;
    fn configure_from_values(&mut self, values, base_path) -> Result<()>;
    async fn test_connection(&self) -> Result<bool>;
    async fn fetch_messages(&self) -> Result<Vec<Message>>;
}
```

### DateRange

All fetching is scoped to a `DateRange` stored as a global singleton after initialization. Run modes:

| Mode | Description |
|------|-------------|
| `today` | Midnight → now |
| `since-last-run` | Last saved run time → now |
| `weekend` | Previous Friday → now |
| `days-N` | N days ago → now |
| `--from / --to` | Custom date range |

## Data Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Registry
    participant Plugin
    participant API as External API
    participant Output

    User->>CLI: work-os sync --plugins github
    CLI->>Registry: fetch_messages_from(["github"])
    Registry->>Plugin: fetch_messages()
    Plugin->>API: HTTP requests
    API-->>Plugin: raw data
    Plugin-->>Registry: Vec<Message>
    Registry-->>CLI: Vec<Message>
    CLI->>Output: terminal / markdown / json
    Output-->>User: formatted results
```

## Directory Structure

```
src/
├── main.rs               — CLI entry point (clap)
├── cli/
│   ├── sync.rs           — sync command
│   ├── stats.rs          — stats command
│   ├── config.rs         — config command
│   └── auth.rs           — auth test command
├── core/
│   ├── message.rs        — Message, MessageType, MessageMetadata, etc.
│   ├── plugin.rs         — Plugin trait + ConfigField schema
│   └── registry.rs       — PluginRegistry
├── generators/
│   └── markdown.rs       — Markdown file generator
├── models/
│   ├── config.rs         — WorkOsConfig (TOML)
│   ├── date_range.rs     — DateRange + RunMode
│   └── state.rs          — WorkOsState (last run tracking)
└── plugins/
    ├── github/           — GitHub plugin
    ├── slack/            — Slack plugin
    ├── jira/             — Jira plugin
    ├── granola/          — Granola plugin
    ├── coralogix/        — Coralogix plugin
    └── google/           — Google plugins (shared OAuth)
        ├── auth.rs       — OAuth2 flow, token refresh, token persistence
        ├── mod.rs        — GooglePlugin (auth-only, owns the OAuth flow)
        ├── calendar/     — Google Calendar plugin
        └── tasks/        — Google Tasks plugin
```


## Configuration Storage

Config is stored in TOML at `~/.config/work-os/config.toml`. Each plugin has an `[plugins.name]` section with `enabled = true/false` and a `values` table.

State (last run timestamps, etc.) is stored at `~/.config/work-os/state.json`.
