use crate::core::task::{PersonRole, Task};
use crate::core::registry::PluginRegistry;
use crate::error::{Result, WorkOsError};
use crate::generators::markdown::{format_duration, get_task_icon, MarkdownGenerator};
use crate::models::config::WorkOsConfig;
use crate::models::date_range::DateRange;
use crate::models::state::WorkOsState;
use crate::plugins::create_registry;
use colored::*;

pub fn setup_workspace(
    run_mode: String,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<(WorkOsConfig, WorkOsState, PluginRegistry)> {
    let config = WorkOsConfig::load()?;
    let state = WorkOsState::load()?;

    let range = DateRange::resolve_date_range(
        from_date.as_deref(),
        to_date.as_deref(),
        run_mode.as_str(),
        &state,
    )?;

    println!("{}", "Date range:".dimmed());
    println!("  {}", range.describe());
    DateRange::init(range);

    let registry = create_registry(&config)?;

    println!();

    Ok((config, state, registry))
}

pub async fn run(
    json_output: bool,
    markdown: bool,
    plugin_filter: Option<Vec<String>>,
    run_mode: String,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<()> {
    let (config, mut state, registry) = setup_workspace(run_mode, from_date, to_date)?;

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

    if markdown {
        let base_path = config.output.base_path.clone();
        let markdown_path = config.output.markdown_path.clone();
        let markdown_generator = MarkdownGenerator::new(base_path.join(markdown_path));
        let output_path = markdown_generator.generate(&all_tasks);
        println!(
            "{} Markdown generated at: {}",
            "✓".green(),
            output_path.unwrap().to_str().unwrap()
        );
    }

    state.update_daily_brief(&DateRange::get().mode)?;

    Ok(())
}

fn print_tasks(tasks: &[Task]) {
    println!("\n{}", "═".repeat(60).dimmed());
    println!("{}", " TASKS ".bold().on_blue().white());
    println!("{}", "═".repeat(60).dimmed());

    tasks.iter().for_each(print_task);

    println!("\n{}", "═".repeat(60).dimmed());
    println!("Total tasks: {}", tasks.len().to_string().bold());
}

fn print_task(task: &Task) {
    let icon = get_task_icon(task);

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
