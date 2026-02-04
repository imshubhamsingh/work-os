# 🎯 Work-OS

> Because juggling Slack, GitHub, and Jira in 47 tabs isn't a productivity strategy.

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

## ✨ Why Should I Care?

- **Stop Context Switching**: All your work items in one place
- **Never Miss Anything**: PRs waiting for review? Slack mentions? We got you
- **Beautiful Output**: Colored terminal UI, JSON for scripts, or Markdown reports
- **Flexible**: Daily briefs, weekly summaries, or custom date ranges
- **Extensible**: Plugin-based architecture (add your own integrations!)

## � Quick Start

```bash
# Build it
cargo build --release

# Set it up
work-os config init

# Add your tokens
work-os config set github token YOUR_TOKEN
work-os config set slack token YOUR_TOKEN

# Get your stuff
work-os sync
```

That's it! Check the `/docs` folder for detailed setup instructions for each platform.

## 💡 Cool Things You Can Do

```bash
# Quick daily standup prep
work-os sync --mode daily

# See what you need to catch up on this week
work-os sync --mode weekly --markdown

# Just check Slack (because we all know why)
work-os sync --plugins slack

# Export everything as JSON for your scripts
work-os sync --json > my-tasks.json
```

## 🤖 My Workflow: AI-Assisted Daily Briefs

Here's where it gets interesting. After `work-os sync` generates the raw markdown, I use **Claude Code with custom templates** to process it into actionable daily briefs.

**My complete stack:**
1. **Work-OS** (Rust CLI) → Syncs tasks from GitHub, Slack, Jira
2. **Claude Code** (AI) → Processes raw data with custom templates
3. **Obsidian** (Markdown) → Stores and organizes my daily briefs

I've set up custom commands (in `.claude/templates/`) that:
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

## 🏗️ How's It Built?

Built with Rust because... well, why not? 🦀

- **Plugins**: Each platform (GitHub, Slack, Jira) is a plugin
- **Tasks**: Everything becomes a unified `Task` model
- **Outputs**: Terminal, JSON, or Markdown - pick your flavor
- **Smart Syncing**: Only fetches what you need based on date ranges

The architecture is clean and modular - adding new platforms is straightforward. Check `/docs` if you want to build a plugin!

## 🎨 Pretty Terminal Output

Work-OS doesn't just dump data - it makes it look good:
- Color-coded priorities
- Source icons (GitHub/Slack/Jira)
- Author info and timestamps  
- Clickable URLs
- Clean formatting

## 🤓 For the Nerds

- Written in Rust (2021 edition)
- Async/await with Tokio
- Clean plugin architecture
- Proper error handling with `thiserror`
- Config in TOML, state tracking, the works

## � Documentation

Head to the `/docs` folder for:
- Detailed setup guides
- Plugin development
- Configuration options
- Architecture deep-dive

## 🤝 Contributing

Got ideas? Found bugs? Want to add a plugin for your favorite tool? PRs welcome!

This is a fun side project that scratches my own itch. If it helps you too, awesome! If you make it better, even more awesome!

## � License

MIT - do whatever you want with it.

---

*Built with ☕ and frustration from too many browser tabs*
