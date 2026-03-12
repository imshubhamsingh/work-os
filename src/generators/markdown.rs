use chrono::Local;

use crate::core::message::{PersonRole, Message, MessageType};
use crate::error::Result;
use crate::models::date_range::DateRange;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct MarkdownGenerator {
    output_path: PathBuf,
}

impl MarkdownGenerator {
    pub fn new(path: PathBuf) -> Self {
        Self { output_path: path }
    }

    pub fn generate(&self, messages: &[Message]) -> Result<PathBuf> {
        let now = Local::now();

        let date_folder = now.format("%Y-%m-%d").to_string();
        let date_path = self.output_path.join(&date_folder);
        std::fs::create_dir_all(&date_path)?;

        let time_stamp = now.format("%H%M").to_string();
        let file_name = format!("sync-{}.md", time_stamp);
        let file_path = date_path.join(&file_name);

        let mut content = self.build_markdown(messages);
        content = self.add_message_statistics(&content, messages);

        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    fn build_markdown(&self, messages: &[Message]) -> String {
        let mut md = String::new();

        let range = DateRange::get();
        md.push_str(&format!("Date range: {}\n\n", range.describe()));

        if messages.is_empty() {
            return md;
        }
        for message in messages {
            self.add_message(&mut md, message)
        }
        md
    }

    fn add_message(&self, md: &mut String, message: &Message) {
        let icon = get_message_icon(message);

        let source = format!("[{}]", message.source.to_uppercase());

        md.push_str(&format!("{} {} {}\n", icon, source, message.title));

        if let Some(description) = &message.description {
            for line in description.lines() {
                md.push_str(&format!("           {}\n", line));
            }
        }

        let mut metadata: Vec<String> = Vec::new();

        let author = find_author(message);
        metadata.push(format!("by @{}", author).to_string());

        metadata.push(Message::format_absolute_time(message.created_at));

        if !metadata.is_empty() {
            md.push_str(&format!("     └─ {}\n", metadata.join(" · ")));
        }

        md.push_str(&format!("     {}\n", message.url));
    }

    fn add_message_statistics(&self, md: &str, messages: &[Message]) -> String {
        let mut md_with_stats = md.to_string();

        let mut source_counts: HashMap<String, i64> = HashMap::new();

        for message in messages {
            *source_counts.entry(message.source.clone()).or_insert(0) += 1;
        }

        if !source_counts.is_empty() {
            md_with_stats.push_str("\n\n## Message Statistics\n");

            for (source, count) in source_counts {
                md_with_stats.push_str(&format!("{}: {} items\n", source, count));
            }
        }

        md_with_stats
    }
}

pub fn find_author(message: &Message) -> String {
    message.people
        .iter()
        .find(|p| p.role == PersonRole::Author)
        .map(|p| format!("{}", p.username))
        .unwrap_or_else(|| "unknown".to_string())
}



pub fn get_message_icon(message: &Message) -> String {
    let icon = match message.message_type {
        MessageType::PullRequest => "🔀",
        MessageType::Issue => "🐛",
        MessageType::Review => "👀",
        MessageType::Message => "💬",
        MessageType::Ticket => "🎫",
        MessageType::Statistics => "📊",
        MessageType::MOM => "🎤",
        MessageType::Canvas => "🖼️",
        MessageType::Coralogix => "🚨",
        MessageType::Other(_) => "📌",
    };
    icon.to_string()
}
