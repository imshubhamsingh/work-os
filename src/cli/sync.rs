use crate::core::task::{PersonRole, Priority, Task, TaskType};
use crate::error::{Result, WorkOsError};
use crate::models::config::WorkOsConfig;
use crate::plugins::github::GithubClient;

use colored::*;

pub async fn run(json_output: bool) -> Result<()> {
    let config = WorkOsConfig::load()?;
    let mut all_tasks: Vec<Task> = Vec::new();

    if let Some(github_config) = config.github {
        println!("{}", "Fetching from GitHub...".dimmed());
        let github_client = GithubClient::new(&github_config)?;
        match github_client.get_all_tasks().await {
            Ok(tasks) => {
                println!("  {} Found {} tasks", "✓".green(), tasks.len());
                all_tasks.extend(tasks);
            }
            Err(e) => {
                println!("  {} GitHub error: {}", "✗".red(), e);
            }
        }
    }

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
