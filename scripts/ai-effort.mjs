#!/usr/bin/env node

/**
 * ai-effort.mjs
 *
 * Hook script that logs AI prompt activity to the work-os raw folder.
 * Works with Claude Code and Cursor (and any tool that supports shell hooks).
 *
 * Supported events:
 *   Claude Code — UserPromptSubmit, SessionEnd
 *     Input: { cwd, session_id, prompt, reason }
 *   Cursor      — beforeSubmitPrompt, sessionEnd
 *     Input: { workspace_roots[], session_id, prompt, duration_ms, reason }
 *
 * The provider is passed as the third argument (process.argv[3]) so the same
 * script serves all tools — just change the argument in each tool's hook config.
 *
 * Time is formatted as 12-hour AM/PM (e.g. "03:01 PM") — consistent across
 * both Claude Code and Cursor since it uses the system clock, not tool input.
 *
 * Installation:
 *   1. Copy to ~/.claude/scripts/ai-effort.mjs
 *   2. chmod +x ~/.claude/scripts/ai-effort.mjs
 *   3. Add hooks to ~/.claude/settings.json and ~/.cursor/hooks.json
 *      (see docs/ai-effort-tracking.md)
 *
 * Log path follows work-os config:
 *   {output.base_path}/{output.markdown_path}/{date}/ai-sessions.jsonl
 */

import fs from 'fs';
import path from 'path';
import os from 'os';

// ---------------------------------------------------------------------------
// User config — set these for your environment
// ---------------------------------------------------------------------------

// Only sessions under this root directory will be logged.
const WORK_ROOT = path.join(os.homedir(), 'Projects');

// Directories to exclude even if they fall under WORK_ROOT.
// Add personal projects, dotfiles, etc.
const EXCLUDED_ROOTS = [
  // Example: path.join(os.homedir(), 'Projects', 'personal'),
];

// ---------------------------------------------------------------------------
// Read work-os config for log path
// ---------------------------------------------------------------------------

function readWorkOsConfig() {
  const configPath = path.join(os.homedir(), '.config', 'work-os', 'config.toml');
  try {
    const content = fs.readFileSync(configPath, 'utf8');
    const basePath = content.match(/base_path\s*=\s*"([^"]+)"/)?.[1];
    const markdownPath = content.match(/markdown_path\s*=\s*"([^"]+)"/)?.[1] ?? 'raw';
    return basePath ? { basePath, markdownPath } : null;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Read hook input from stdin
// ---------------------------------------------------------------------------

let input = {};
try {
  const raw = fs.readFileSync('/dev/stdin', 'utf8');
  input = JSON.parse(raw);
} catch {
  process.exit(0);
}

// Claude Code sends `cwd`; Cursor sends `workspace_roots` (array)
const cwd = input.cwd ?? input.workspace_roots?.[0] ?? '';

// ---------------------------------------------------------------------------
// Filter: only log company project sessions
// ---------------------------------------------------------------------------

const isWorkProject = cwd.startsWith(WORK_ROOT);
const isExcluded = EXCLUDED_ROOTS.some((root) => cwd.startsWith(root));

if (!isWorkProject || isExcluded) {
  process.exit(0);
}

// ---------------------------------------------------------------------------
// Resolve log path from work-os config
// ---------------------------------------------------------------------------

const config = readWorkOsConfig();
if (!config) process.exit(0);

const now = new Date();
const date = now.toISOString().slice(0, 10);
const time = now.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: true });
const event = process.argv[2] ?? 'prompt';    // "prompt" or "session_end"
const provider = process.argv[3] ?? 'claude'; // "claude", "cursor", etc.
const project = path.basename(cwd);

// ---------------------------------------------------------------------------
// Build record
// ---------------------------------------------------------------------------

const record = {
  event,
  provider,
  project,
  session_id: input.session_id ?? '',
  time,
  date,
};

if (event === 'prompt' && input.prompt) {
  record.prompt = input.prompt;
}

if (event === 'session_end') {
  if (input.reason) record.reason = input.reason;
  if (input.duration_ms) record.duration_ms = input.duration_ms;
}

// ---------------------------------------------------------------------------
// Append to JSONL log
// ---------------------------------------------------------------------------

const logDir = path.join(config.basePath, config.markdownPath, date);
const logFile = path.join(logDir, 'ai-sessions.jsonl');

fs.mkdirSync(logDir, { recursive: true });
fs.appendFileSync(logFile, JSON.stringify(record) + '\n');
