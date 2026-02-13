# 🎯 Work-OS

> Because juggling Slack, GitHub, Jira, and meeting notes in 47 tabs isn't a productivity strategy.

A Rust CLI that syncs your work from everywhere, then uses AI (Claude Code) to turn the chaos into clean markdown briefs. Your personal productivity system, automated. (I use Obsidian to view them, but it's just markdown files!)

## 🤔 What's This?

Ever had that moment where you're like "Wait, did I miss reviewing that PR?" or "Which Slack thread was that in?" Yeah, me too. 

Work-OS is a Rust CLI that pulls together your:
- 💬 Slack messages, mentions, and DMs
- 🔀 GitHub PRs, issues, and reviews
- 🎫 Jira tickets and sprints

...and gives you a clean, unified view. One command, all your tasks. Simple.

## 🎓 Why I Built This

Two reasons, really:
1. **Learn Rust**: I wanted to dive into Rust properly, and what better way than building something real?
2. **Solve My Own Problem**: I was tired of switching between 10 tabs every morning to figure out what I needed to work on. So I built a tool to fix that.

This started as a learning project and turned into something I actually use every day. If it helps you too, that's a bonus!

### The philosophy

The core idea behind this project is using AI as a **learning tool**, not a replacement for learning. I didn't want AI to just build things for me. I wanted to use it to understand *how* things work: why Rust ownership works the way it does, how async runtimes are structured, how to design a plugin system. AI accelerates that understanding; it doesn't skip it.

Could I have built this faster in TypeScript? Absolutely. Probably a weekend project, maybe less with AI doing the heavy lifting. That was never the point.

The productivity problem at work was the real-world constraint that kept the project grounded. Without a genuine problem to solve, it's easy for learning projects to stay theoretical.

## ✨ Why Should I Care?

- **Stop Context Switching**: All your work items in one place
- **Never Miss Anything**: PRs waiting for review? Slack mentions? We got you
- **Track AI Collaboration**: See exactly how much AI is helping with your code
- **Beautiful Output**: Colored terminal UI, JSON for scripts, or Markdown reports
- **Flexible**: Daily briefs, weekly summaries, or custom date ranges
- **Extensible**: Plugin-based architecture (add your own integrations!)

## 📦 Installation

Download the latest macOS binary from [Releases](../../releases/latest):

```bash
# Apple Silicon (M1/M2/M3)
curl -L https://github.com/imshubhamsingh/work-os/releases/latest/download/work-os-macos-arm64 -o work-os
chmod +x work-os && sudo mv work-os /usr/local/bin/

# Intel Mac
curl -L https://github.com/imshubhamsingh/work-os/releases/latest/download/work-os-macos-x64 -o work-os
chmod +x work-os && sudo mv work-os /usr/local/bin/
```

Or build from source:

```bash
cargo install --path .
```

## 🚀 Quick Start

```bash
# Build it
cargo build --release

# Set it up
work-os config init

# Configure GitHub
work-os config set github token YOUR_GITHUB_TOKEN
work-os config set github username YOUR_USERNAME

# Configure Slack
work-os config set slack token YOUR_SLACK_TOKEN

# Configure Jira
work-os config set jira domain company.atlassian.net
work-os config set jira email your-email@company.com
work-os config set jira token YOUR_JIRA_API_TOKEN

# Get your stuff
work-os sync
```

That's it! See the plugin setup guides: [GitHub](docs/plugins/github.md) · [Slack](docs/plugins/slack.md) · [Jira](docs/plugins/jira.md)

## 💡 Cool Things You Can Do

```bash
# Quick daily standup prep
work-os sync --mode daily

# See what you need to catch up on this week
work-os sync --mode weekly --markdown

# Just check Slack (because we all know why)
work-os sync --plugins slack

# Check your Jira tickets and sprint status
work-os sync --plugins jira

# GitHub + Jira only (the developer combo)
work-os sync --plugins github,jira

# Export everything as JSON for your scripts
work-os sync --json > my-tasks.json

# Track your AI-assisted coding stats
work-os stats --type ai-code

# Get weekly AI code usage statistics
work-os stats --type ai-code --mode weekly
```

## 🤖 My Workflow: AI-Assisted Daily Briefs

Here's where it gets interesting. After `work-os sync` generates the raw markdown, I use **Claude Code with custom templates** to process it into actionable daily briefs.

**My complete stack:**
1. **Work-OS** (Rust CLI) → Syncs tasks from GitHub, Slack, and Jira
2. **Claude Code** (AI) → Processes raw data with custom templates
3. **Obsidian** (Markdown) → Stores and organizes my daily briefs, weekly reports and follow-ups

I've set up custom commands (in `prompts`) that:
- Take the raw sync data from all platforms
- Extract actionable items and categorize them (Must Do, Reviews, Follow-ups)
- Detect release-critical PRs and blockers
- Track carryovers from previous days
- Generate a clean, prioritized daily brief in my Obsidian vault

**Quick example:**
```bash
# Sync all your work
work-os sync --markdown

# Then in Claude Code, run the custom template
/work-os-today

# Opens in Obsidian with clean, actionable brief
```

This combo lets me start each morning with a crystal-clear view of what matters, all stored in my Obsidian vault for easy reference and searching. You can use any markdown editor you prefer, but I've found Obsidian perfect for this workflow!

The templates are in the repo if you want to adapt them for your own workflow.

## 📊 AI Code Usage Tracking

Ever wonder how much AI is actually helping you code? Work-OS tracks AI-assisted development through GitHub commit analysis:

**What it detects:**
- 🤖 **Explicit AI Attribution**: Commits with `Co-Authored-By: Claude` or similar
- 💥 **Large Code Bursts**: Commits with >1000 lines (often AI-generated)
- 📝 **Commit Patterns**: Long descriptive vs. short simple messages
- 🔀 **Smart Filtering**: Excludes merge commits from AI scoring

**Commit-level breakdown:**
```bash
work-os stats --type ai-code

# Example output:
PR: feat/new-dashboard (#234)
  AI Score: 75% (3/4 commits)

  Commit Details:
    - abc1234: +1250 -45 (80%) - AI: Large burst
    - def5678: +120 -30 (100%) - AI: Claude (explicit)
    - ghi9012: +45 -12 (0%) - Human: Simple commit
    - merge: (skipped) - Merge commit
```

This helps you understand your AI collaboration patterns and track productivity trends over time.

## 🏗️ How's It Built?

Built with Rust because... well, why not? 🦀

- **Plugins**: Each platform (GitHub, Slack, Jira) is a plugin
- **Messages**: Everything becomes a unified `Message` model
- **Outputs**: Terminal, JSON, or Markdown - pick your flavor
- **Smart Syncing**: Only fetches what you need based on date ranges

The architecture is clean and modular - adding new platforms is straightforward. See [Building a Plugin](docs/plugins/building-a-plugin.md).

## 🎨 Pretty Terminal Output

Work-OS doesn't just dump data - it makes it look good:
- Color-coded priorities
- Source icons (GitHub/Slack/Jira)
- Author info and timestamps
- Clickable URLs
- Clean formatting

> **Note:** The current terminal output is pretty basic — it's essentially a styled dump. A proper interactive TUI (think panels, navigation, filtering) is something I want to build from scratch as a learning project.

## 🤓 For the Nerds

- Written in Rust (2021 edition)
- Async/await with Tokio
- Clean plugin architecture
- Proper error handling with `thiserror`
- Config in TOML, state tracking, the works

## 📖 Documentation

| Doc | Description |
|-----|-------------|
| [My Workflow](docs/workflow.md) | How I actually use this day to day |
| [Architecture](docs/architecture.md) | System overview, data flow, directory structure |
| [Configuration](docs/configuration.md) | Full config file reference with examples |
| [GitHub Plugin](docs/plugins/github.md) | Setup, AI stats, token scopes |
| [Slack Plugin](docs/plugins/slack.md) | Setup, OAuth scopes, what gets fetched |
| [Jira Plugin](docs/plugins/jira.md) | Setup, JQL filters, priority mapping |
| [Building a Plugin](docs/plugins/building-a-plugin.md) | How to add a new integration |

## 🔮 Planned Integrations

Plugins I want to add in the future:
- **Google Calendar** — pull upcoming meetings and deadlines into your daily brief
- **Google Docs / Sheets** — surface recently edited documents and spreadsheets
- **Figma** — track design file updates and comments
- **Notion** — sync tasks, pages, and databases

## 💡 Where This Could Go

Once you have weeks/months of your own work history sitting in markdown files, the daily brief is just the start. Some ideas I want to explore:

**Personal context retrieval**
- Before picking up a ticket, surface everything you've touched that's related: past PRs, the Slack threads where the design was debated, the meeting where the decision was made
- "I've worked on something like this before" is useful. Knowing exactly what and where is better.

**Reflection and growth**
- Look back across weeks or months: where is your time actually going, which areas are you improving in, what keeps coming back as a blocker
- Not as a performance report, just honest signal for yourself

**Smarter meeting prep**
- Given a calendar event, pull up everything relevant: past discussions with those people, related tickets, your own notes from last time
- Walk in with context instead of scrambling to remember

**Personal knowledge base**
- Build up a searchable record of decisions you've made and why, written from actual data rather than from memory
- Useful six months later when you're asking "why did we do it this way"

**LLM agnostic**
- Right now the AI workflow is built around Claude. That's fine until you hit a rate limit and have to wait. I want to make it easy to swap providers (Gemini, GPT, local models) without rewriting the prompts or templates.
- Beyond just fallback, it's also a playground. Same prompt, same data, different models — a good way to compare outputs and get a feel for where each model actually differs in practice.
- The goal is: one config change, everything still works.

**Building my own RAG system**
- All this synced data is sitting in markdown files — structured, dated, and searchable. That's a pretty good starting point for a retrieval-augmented generation system.
- Rather than plugging into an existing RAG framework (like [qmd](https://github.com/tobi/qmd)), I want to build one from scratch here to actually understand how embedding, chunking, and retrieval work. Work-OS gives a real dataset to experiment on, which makes it less theoretical.

The common thread: the raw material is already there. It's just scattered across tools and mostly forgotten after the week ends.

## 🤝 Contributing

Look, it's Rust — not exactly Go in terms of "everyone and their dog sends PRs". The borrow checker alone has probably scared off half the potential contributors. Totally fine by me, this is a personal project first.

That said, if you're one of the brave souls who enjoys fighting the compiler for fun (respect), found a bug, or want to add a plugin for your favourite tool — PRs are very welcome. If it helps you too, awesome. If you make it better, even more awesome.

## � License

MIT - do whatever you want with it.

---

*Built with ☕ and frustration from too many browser tabs*

## ⚠️ A Note on Workplace Use

This tool connects to platforms like Slack, GitHub, and Jira, which means it interacts with data owned by your organization. If you plan to use this at work, **get explicit permission from your team or company before setting it up**. Even if the tool only reads data you already have access to, most organizations have policies around third-party tools, API token usage, and data handling. A quick conversation with your manager or security team goes a long way. Don't assume it's fine just because it's technically possible.
