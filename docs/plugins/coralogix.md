# Coralogix Plugin

Fetches production ERROR-severity logs from Coralogix via the DataPrime REST API, writes them to a daily JSONL file, and surfaces a per-application error summary with trend analysis.

## Setup

```bash
work-os config set coralogix api_key YOUR_LOGS_QUERY_API_KEY
work-os config set coralogix domain https://yourteam.coralogix.com
work-os config set coralogix application_names my-backend-service

# Multiple applications (comma-separated)
work-os config set coralogix application_names "my-backend-service,my-worker-service"
```

## Config reference

| Field | Required | Description |
|-------|----------|-------------|
| `api_key` | Yes | Coralogix Logs Query API key (see below) |
| `domain` | Yes | Your Coralogix dashboard URL, e.g. `https://yourteam.coralogix.com` |
| `application_names` | Yes | One or more Coralogix application names to query |

### Getting the API key

In Coralogix: **Data Flow → API Keys → Logs Query Key**

This is a read-only key scoped to log queries — not the Send-Your-Data key.

## What It Fetches

The plugin queries:

```
source logs
| filter $m.severity == ERROR
| filter $l.applicationname == '<your-app>'
| filter $d.logRecord.attributes.environment == 'production'
| orderby $m.timestamp desc
```

- Only `ERROR` severity logs
- Only the application names you configured
- Only the `production` environment
- Scoped to the date range of the current sync run

## Output

### JSONL file

All records are written (append-only, deduped by `logid`) to:

```
{output.base_path}/{output.markdown_path}/{DATE}/coralogix.jsonl
```

Each line is a flat JSON record:

```json
{
  "logid": "abc123",
  "timestamp": "2026-03-12T08:45:00.123",
  "application_name": "my-backend-service",
  "severity": "Error",
  "body": "Failed to process payment",
  "error": "Connection timeout: downstream service unreachable",
  "service": "payment-service",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "environment": "production",
  "url": "https://yourteam.coralogix.com/#/query-new/logs?permalink=true&startTime=...&logId=abc123"
}
```

### Summary message

One `MessageType::Coralogix` message is produced per application, containing a markdown analysis:

```
## 🚨 Production Errors · my-backend-service (142 errors ↑ from 98 prev (vs 2026-03-11))

### 🔁 Recurring — needs attention

| Count | Trend | Error | Link |
|-------|-------|-------|------|
| 87x | ↑ 45→87 | Failed to process payment | [→](...) |
| 23x | 🆕 | Timeout calling downstream service | [→](...) |

### ⚠️ One-off Concerns

- `Invalid request payload` (3x 🆕) — [→](...)

### 🆕 New Since Last Run (2026-03-11)

- `Timeout calling downstream service` (23x) — [→](...)

### ✅ Resolved Since Last Run

- `Connection pool exhausted` — was 12x, gone today
```

Trend comparison looks at the most recent previous date folder that has a `coralogix.jsonl`.

## Multi-day syncs

If you run with `--mode weekly` or a custom date range, all records are written to a single JSONL file dated at the **end** of the range (the day you ran the sync). The trend comparison still uses the most recent prior day's file.

## Deduplication

Records are deduped by `logid` on append. Running the same sync twice won't double-count.
