# Google Plugin

Work-OS has three Google plugins that share a single OAuth2 connection:

| Plugin | ID | What it fetches |
|--------|----|----|
| **Google** | `google` | Auth only — no messages |
| **Google Calendar** | `google_calendar` | Events from your primary calendar |
| **Google Tasks** | `google_tasks` | Incomplete tasks across all task lists |

## How Auth Works

All three plugins share one OAuth2 token. You authenticate once with `work-os auth google` and both Calendar and Tasks use that token automatically.

The token is stored in `~/.work-os/config.toml` under `[plugins.google]`:

```toml
[plugins.google]
access_token = "ya29...."
refresh_token = "1//..."
expires_at = 1749999999
```

The access token is refreshed automatically when it expires (within 5 minutes of expiry).

## Setup

There are three parts to this:

1. Create a Google Cloud project and register Work-OS as an OAuth app (one-time, a few minutes)
2. Embed the app credentials at build time
3. Authenticate your Google account with `work-os auth google`

---

### Part 1: Create a Google Cloud project

This gives you a client ID and secret that identifies Work-OS as an OAuth app. You do this once. The credentials don't contain any of your personal data — they just tell Google "this is the Work-OS app."

**Step 1 — Create a project**

Go to [console.cloud.google.com](https://console.cloud.google.com).

In the top bar, click the project dropdown (it may say "Select a project" or show an existing project name). Click **New Project**. Give it any name — "Work OS" works fine. Click **Create**.

Make sure the new project is selected in the top bar before continuing.

**Step 2 — Enable the APIs**

In the left sidebar, go to **APIs & Services > Library**.

Search for and enable each of these:

- **Google Calendar API** — click it, then click **Enable**
- **Google Tasks API** — click it, then click **Enable**

**Step 3 — Configure the OAuth consent screen**

Go to **APIs & Services > OAuth consent screen**.

You'll be asked to choose a user type. Select **External** (this is for personal use, not a Workspace org). Click **Create**.

Fill in the required fields:

| Field | What to enter |
|-------|--------------|
| App name | `Work OS` |
| User support email | Your Google account email |
| Developer contact email | Your Google account email |

Leave everything else blank. Click **Save and Continue**.

On the **Scopes** step, click **Save and Continue** without adding anything — scopes are requested at runtime, not configured here.

On the **Test users** step, click **Add Users** and add your own Google account email. This is required while the app is in testing mode — only listed users can authenticate. Click **Save and Continue**, then **Back to Dashboard**.

> You don't need to publish the app or go through Google verification. Keeping it in testing mode is fine for personal use — just make sure your account is in the test users list.

**Step 4 — Create OAuth2 credentials**

Go to **APIs & Services > Credentials**. Click **+ Create Credentials** at the top, then choose **OAuth client ID**.

Fill in:

| Field | What to enter |
|-------|--------------|
| Application type | **Desktop app** |
| Name | `Work OS CLI` |

Click **Create**.

A dialog will appear with your **Client ID** and **Client Secret**. Copy both — you'll need them in the next step.

You can always come back to this page (Credentials) to view them again.

---

### Part 2: Embed credentials at build time

Google credentials are embedded into the binary at compile time, not stored in the config file. This is because they're app-level credentials — they identify your OAuth app, not your personal account.

Create (or open) `.cargo/config.toml` in the repo root. This file is gitignored:

```toml
[env]
GOOGLE_CLIENT_ID = "123456789-abc.apps.googleusercontent.com"
GOOGLE_CLIENT_SECRET = "GOCSPX-your-secret-here"
```

Replace the values with what you copied in Step 4.

Then build:

```bash
cargo build --release
```

You must rebuild any time you change these values. The credentials are baked into the binary.

---

### Part 3: Authenticate your Google account

```bash
work-os auth google
```

This will:

1. Open your browser to Google's OAuth consent screen
2. Ask you to select your Google account and grant Calendar and Tasks access
3. Redirect back to a local callback server
4. Save the token to `~/.work-os/config.toml` under `[plugins.google]`

You only need to do this once. The access token refreshes automatically when it expires.

If the browser doesn't open, the terminal will print a URL you can paste manually.

---

### Part 4: Enable the plugins in config

Add this to `~/.work-os/config.toml`:

```toml
[plugins.google_calendar]
enabled = true

[plugins.google_tasks]
enabled = true
```

Or use the interactive setup:

```bash
work-os config init google_calendar
work-os config init google_tasks
```

## Permissions (OAuth2 Scopes)

| Scope | Plugin | Why |
|-------|--------|-----|
| `https://www.googleapis.com/auth/calendar.readonly` | Google Calendar | Read events from your primary calendar |
| `https://www.googleapis.com/auth/tasks.readonly` | Google Tasks | Read task lists and tasks |

Both scopes are read-only. Work-OS never writes to your calendar or tasks.

---

## Google Calendar

### What it fetches

Events from your primary calendar within the sync date range, extended by `upcoming_days` into the future.

- Cancelled events are excluded
- All-day events are included
- Status is derived from event time: `Open` (future), `InProgress` (now), `Done` (past)
- Attendee RSVP status is shown (accepted / declined / tentative / awaiting)
- Meeting links (Google Meet / conference data) are extracted as the message URL
- Event description is included (truncated to 500 characters, HTML stripped)

### Configuration

```toml
[plugins.google_calendar]
enabled = true
upcoming_days = 3  # how many days ahead to fetch beyond today (default: 3)

[plugins.google_calendar.colors]
# Map Google Calendar color names to your own labels
Sage = "Focus time"
Tomato = "Interviews"
Blueberry = "1:1s"
```

`upcoming_days` extends the end of the sync range forward. For example, if you run `work-os sync --mode daily`, the calendar plugin will fetch today's events plus the next 3 days.

#### Color labels

Google Calendar lets you assign a color to each event. You can map those color names to your own labels in `[plugins.google_calendar.colors]`. Google's color names are: `Tomato`, `Flamingo`, `Tangerine`, `Banana`, `Sage`, `Basil`, `Peacock`, `Blueberry`, `Lavender`, `Grape`, `Graphite`.

If a color is not in your map, the Google color name is shown as-is.

### Priority mapping

| Event type | Priority |
|------------|----------|
| Focus Time | `High` |
| Everything else | `Unknown` |

### CLI usage

```bash
# Sync calendar only
work-os sync --plugins google_calendar

# Sync calendar + tasks
work-os sync --plugins google_calendar,google_tasks
```

---

## Google Tasks

### What it fetches

All incomplete tasks across all your task lists. Completed tasks are excluded.

- Tasks from every task list in your account
- Subtasks are included and marked with "Subtask" in the description
- Due date is shown if set
- Notes are included in the description
- Priority is derived from due date proximity

### Priority mapping

| Due date | Priority |
|----------|----------|
| Overdue or due today | `High` |
| Due within 3 days | `Medium` |
| Due later or no due date | `Low` |

### Configuration

No additional config needed beyond the shared OAuth token.

```toml
[plugins.google_tasks]
enabled = true
```

### CLI usage

```bash
# Sync tasks only
work-os sync --plugins google_tasks
```

---

## Re-authenticating

```bash
# Force re-authentication (e.g. after revoking access or changing scopes)
work-os auth google --force
```

## Troubleshooting

**"No OAuth token found. Run: work-os auth google"**
Run `work-os auth google` to authenticate.

**"GOOGLE_CLIENT_ID not set"**
The binary was built without credentials. Add them to `.cargo/config.toml` and rebuild (`cargo build --release`).

**"Access blocked: Work OS has not completed the Google verification process"**
Your account isn't in the test users list. Go to Google Cloud Console > APIs & Services > OAuth consent screen > Test users, and add your Google account email. You do not need to publish the app.

**"Token refresh failed" or auth stops working after a while**
This can happen if you revoke access from your Google Account or if the refresh token expires (Google expires refresh tokens after 6 months of inactivity). Run `work-os auth google --force` to re-authenticate.

**Calendar returns no events**
Check that `upcoming_days` covers the range you expect. Also verify your primary calendar has events in the sync window. Work-OS only fetches from the primary calendar — events on secondary/shared calendars are not included.

**Tasks returns no items**
All tasks in your lists may be completed. Completed tasks are intentionally excluded.

**Authentication timed out**
The OAuth callback waits up to 2 minutes. If you didn't complete the browser flow in time, run `work-os auth google` again.
