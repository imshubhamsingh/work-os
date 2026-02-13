use crate::core::message::{PersonRole, Message};
use crate::core::registry::PluginRegistry;
use crate::error::{Result, WorkOsError};
use crate::generators::markdown::{get_message_icon, MarkdownGenerator};
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

    println!("{}", "Fetching messages...".dimmed());

    let all_messages = match plugin_filter {
        Some(plugins) => registry.fetch_messages_from(&plugins).await?,
        None => registry.fetch_messages_from(&registry.list_ids()).await?,
    };

    if all_messages.is_empty() {
        println!("\n{}", "No messages found!".yellow());
        return Ok(());
    }

    if json_output {
        let json = serde_json::to_string_pretty(&all_messages)
            .map_err(|e| WorkOsError::Config(e.to_string()))?;
        println!("{}", json);
    } else {
        print_messages(&all_messages);
    }

    if markdown {
        let base_path = config.output.base_path.clone();
        let markdown_path = config.output.markdown_path.clone();
        let markdown_generator = MarkdownGenerator::new(base_path.join(markdown_path));
        let output_path = markdown_generator.generate(&all_messages);
        println!(
            "{} Markdown generated at: {}",
            "✓".green(),
            output_path.unwrap().to_str().unwrap()
        );
    }

    state.update_daily_brief(&DateRange::get().mode)?;

    Ok(())
}

fn print_messages(messages: &[Message]) {
    println!("\n{}", "═".repeat(60).dimmed());
    println!("{}", " TASKS ".bold().on_blue().white());
    println!("{}", "═".repeat(60).dimmed());

    messages.iter().for_each(print_message);

    println!("\n{}", "═".repeat(60).dimmed());
    println!("Total messages: {}", messages.len().to_string().bold());
}

fn print_message(message: &Message) {
    let icon = get_message_icon(message);

    let source = format!("[{}]", message.source.to_uppercase()).dimmed();

    println!("{} {} {}", icon, source, message.title.bold());

    if let Some(description) = &message.description {
        for line in description.lines() {
            println!("           {}", line.dimmed());
        }
    }

    let mut metadata: Vec<String> = Vec::new();

    if let Some(author) = message.people.iter().find(|p| p.role == PersonRole::Author) {
        metadata.push(format!("by @{}", author.username).dimmed().to_string());
    }

    metadata.push(Message::format_absolute_time(message.created_at).dimmed().to_string());

    if !metadata.is_empty() {
        println!("     └─ {}", metadata.join(" · "));
    }

    println!("     {}", message.url.dimmed());
}
