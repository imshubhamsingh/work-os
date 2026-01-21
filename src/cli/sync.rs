use crate::core::task::{PersonRole, Priority, Task, TaskType};
use crate::error::{Result, WorkOsError};
use crate::models::config::WorkOsConfig;
use crate::plugins::create_registry;

use chrono::{DateTime, Utc};
use colored::*;

pub async fn run(json_output: bool, plugin_filter: Option<Vec<String>>) -> Result<()> {
    let config = WorkOsConfig::load()?;
    let registry = create_registry(&config)?;

    println!("{}", "Plugins:".dimmed());

    for plugin in registry.get_all() {
        let meta = plugin.metadata();
        let status = if plugin.is_configured() {
            "✓ configured".green()
        } else {
            "✗ not configured".red()
        };

        println!("  {} {} {}", meta.icon, meta.name, status);
    }

    println!();

    println!("{}", "Fetching tasks...".dimmed());

    let all_tasks = match plugin_filter {
        Some(plugins) => registry.fetch_tasks_from(&plugins).await?,
        None => registry.fetch_tasks_from(&registry.list_ids()).await?,
    };

    if all_tasks.is_empty() {
        println!("\n{}", "No tasks found!".yellow());
        return Ok(());
    }

    if json_output {
        let json = serde_json::to_string_pretty(&all_tasks)
            .map_err(|e| WorkOsError::Config(e.to_string()))?;
        println!("{}", json);
    } else {
        print_tasks(&all_tasks);
    }

    Ok(())
}

fn print_tasks(tasks: &[Task]) {
    println!("\n{}", "═".repeat(60).dimmed());
    println!("{}", " TASKS ".bold().on_blue().white());
    println!("{}", "═".repeat(60).dimmed());

    let critical: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.priority == Priority::Critical)
        .collect();
    let high: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.priority == Priority::High)
        .collect();
    let medium: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.priority == Priority::Medium)
        .collect();
    let low: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.priority == Priority::Low)
        .collect();

    let rest: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.priority == Priority::Unknown)
        .collect();

    if !critical.is_empty() {
        println!("\n{}", "🔴 CRITICAL".red().bold());
        for task in critical {
            print_task(task);
        }
    }

    if !high.is_empty() {
        println!("\n{}", "🟠 HIGH PRIORITY".yellow().bold());
        for task in high {
            print_task(task);
        }
    }

    if !medium.is_empty() {
        println!("\n{}", "🟡 MEDIUM PRIORITY".yellow().bold());
        for task in medium {
            print_task(task);
        }
    }

    if !low.is_empty() {
        println!("\n{}", "🟢 LOW PRIORITY".green().bold());
        for task in low {
            print_task(task);
        }
    }

    if !rest.is_empty() {
        println!("\n{}", "UNKNOWN PRIORITY".yellow().bold());
        for task in rest {
            print_task(task);
        }
    }

    println!("\n{}", "═".repeat(60).dimmed());
    println!("Total tasks: {}", tasks.len().to_string().bold());
}

fn print_task(task: &Task) {
    let icon = match task.task_type {
        TaskType::PullRequest => "🔀",
        TaskType::Issue => "🐛",
        TaskType::Review => "👀",
        TaskType::Message => "💬",
        TaskType::Ticket => "🎫",
        TaskType::Statistics => "📊",
        TaskType::Other(_) => "📌",
    };

    let source = format!("[{}]", task.source.to_uppercase()).dimmed();

    println!("{} {} {}", icon, source, task.title.bold());

    if let Some(description) = &task.description {
        for line in description.lines() {
            println!("           {}", line.dimmed());
        }
    }

    let mut metadata: Vec<String> = Vec::new();

    if let Some(author) = task.people.iter().find(|p| p.role == PersonRole::Author) {
        metadata.push(format!("by @{}", author.username).dimmed().to_string());
    }

    metadata.push(format_duration(task.created_at).dimmed().to_string());

    if !metadata.is_empty() {
        println!("     └─ {}", metadata.join(" · "));
    }

    println!("     {}", task.url.dimmed());
}

fn format_duration(date: DateTime<Utc>) -> String {
    let mut duration_in_minutes = (Utc::now().timestamp() - date.timestamp()) / 60;

    let minutes_in_year = 60 * 24 * 365;
    let minutes_in_month = 60 * 24 * 30;
    let minutes_in_week = 60 * 24 * 7;
    let minutes_in_day = 60 * 24;
    let minutes_in_hour = 60;

    let year = duration_in_minutes / minutes_in_year;
    duration_in_minutes %= minutes_in_year;
    let month = duration_in_minutes / minutes_in_month;
    duration_in_minutes %= minutes_in_month;
    let week = duration_in_minutes / minutes_in_week;
    duration_in_minutes %= minutes_in_week;
    let day = duration_in_minutes / minutes_in_day;
    duration_in_minutes %= minutes_in_day;
    let hour = duration_in_minutes / minutes_in_hour;
    duration_in_minutes %= minutes_in_hour;
    let minute = duration_in_minutes;

    let mut parts = Vec::new();
    if year > 0 {
        parts.push(format!("{}y", year));
    }
    if month > 0 {
        parts.push(format!("{}m", month));
    }
    if week > 0 {
        parts.push(format!("{}w", week));
    }
    if day > 0 {
        parts.push(format!("{}d", day));
    }
    if hour > 0 {
        parts.push(format!("{}h", hour));
    }
    if minute > 0 {
        parts.push(format!("{}m", minute));
    }

    if parts.is_empty() {
        "just now".to_string()
    } else {
        format!("{} ago", parts.join(" "))
    }
}
