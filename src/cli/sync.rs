use crate::core::task::{PersonRole, Priority, Task, TaskType};
use crate::error::{Result, WorkOsError};
use crate::models::config::WorkOsConfig;
use crate::plugins::create_registry;

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
        TaskType::Other(_) => "📌",
    };

    let source = format!("[{}]", task.source.to_uppercase()).dimmed();

    println!("{} {} {}", icon, source, task.title.bold());

    let mut metadata: Vec<String> = Vec::new();

    if let Some(author) = task.people.iter().find(|p| p.role == PersonRole::Author) {
        metadata.push(format!("by @{}", author.username).dimmed().to_string());
    }

    let age = chrono::Utc::now() - task.created_at;
    let age_str = if age.num_days() > 30 {
        format!("created {}m ago", (age.num_days() / 30))
    } else if age.num_days() > 0 {
        format!("created {}d ago", age.num_days())
    } else if age.num_hours() > 0 {
        format!("created {}h", age.num_hours())
    } else if age.num_minutes() > 0 {
        format!("created {}m", age.num_minutes())
    } else {
        format!("created just now")
    };

    metadata.push(age_str.dimmed().to_string());

    if !metadata.is_empty() {
        println!("     └─ {}", metadata.join(" · "));
    }

    println!("     {}", task.url.dimmed());
}
