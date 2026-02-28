# AI Effort Tracking

Track how many prompts you send to Claude Code per day, per project — logged automatically alongside your work-os sync files.

## What it tracks

Every time you submit a prompt in Claude Code from a work project directory, a record is appended to:

```
{output.base_path}/{output.markdown_path}/{date}/ai-sessions.jsonl
```

This follows the same path structure as your sync files (see [configuration](./configuration.md)).

Each record looks like:

```json
{"event":"prompt","project":"my-project","session_id":"abc123","time":"03:01 PM","date":"2026-02-28","prompt":"How do I implement the vendor dropdown?"}
```

On session close, a final record is appended with `"event":"session_end"` and the exit reason.

---

## Why JSONL

One JSON object per line. The file is append-only — the hook script writes one line per event and never reads or rewrites the file. Your `work-os-today` skill reads it at EOD to compute prompt counts per project and surface them in the AI Direction Log.

---

## Setup

### 1. Copy the script

```bash
cp scripts/ai-effort.mjs ~/.claude/scripts/ai-effort.mjs
chmod +x ~/.claude/scripts/ai-effort.mjs
```

### 2. Configure filtering

Open `~/.claude/scripts/ai-effort.mjs` and set:

```js
// Root directory containing your work projects
const WORK_ROOT = path.join(os.homedir(), 'Projects');

// Directories to exclude even if under WORK_ROOT
const EXCLUDED_ROOTS = [
  path.join(os.homedir(), 'Projects', 'personal'),
];
```

Only sessions in directories under `WORK_ROOT` (and not in `EXCLUDED_ROOTS`) will be logged. Sessions in personal projects, dotfiles, or unrelated directories are silently skipped.

### 3. Add hooks to Claude Code

Add the following to `~/.claude/settings.json`:

```json
"hooks": {
  "UserPromptSubmit": [
    {
      "hooks": [
        {
          "type": "command",
          "command": "node ~/.claude/scripts/ai-effort.mjs prompt",
          "async": true
        }
      ]
    }
  ],
  "SessionEnd": [
    {
      "hooks": [
        {
          "type": "command",
          "command": "node ~/.claude/scripts/ai-effort.mjs session_end",
          "async": true
        }
      ]
    }
  ]
}
```

`async: true` means the hook runs after the event fires and never blocks Claude Code.

### 4. Verify

Test the script manually:

```bash
echo '{"cwd":"/your/projects/my-project","session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"hello"}' \
  | node ~/.claude/scripts/ai-effort.mjs prompt
```

Then check the log was written:

```bash
cat ~/.config/work-os  # find your base_path
# then:
cat {base_path}/raw/$(date +%Y-%m-%d)/ai-sessions.jsonl
```

---

## Record schema

| Field | Event | Description |
|-------|-------|-------------|
| `event` | all | `"prompt"` or `"session_end"` |
| `project` | all | Directory name of the project (`basename` of `cwd`) |
| `session_id` | all | Unique session identifier from Claude Code |
| `time` | all | Local time with AM/PM, e.g. `03:01 PM` |
| `date` | all | ISO date, e.g. `2026-02-28` |
| `prompt` | prompt only | The exact text submitted by the user |
| `reason` | session_end only | Exit reason: `exit`, `timeout`, etc. |
| `duration_ms` | session_end only | Session duration in milliseconds (Cursor only) |

---

## How work-os-today uses it

At EOD generation, `work-os-today` reads `ai-sessions.jsonl` and groups records by `project`. For each project it computes:

- **Prompt count** — number of `"event":"prompt"` records
- **Sessions** — number of distinct `session_id` values
- **First / last prompt time** — from `time` field

These feed the `### 🧠 AI Direction Log` table in the EOD reflection. The prompt text itself is available for qualitative review directly in the JSONL file.
