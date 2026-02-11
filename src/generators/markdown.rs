use chrono::Local;

use crate::core::task::{PersonRole, Task, TaskType};
use crate::error::Result;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct MarkdownGenerator {
    output_path: PathBuf,
}

impl MarkdownGenerator {
    pub fn new(path: PathBuf) -> Self {
        Self { output_path: path }
    }

    pub fn generate(&self, tasks: &[Task]) -> Result<PathBuf> {
        let now = Local::now();

        let date_folder = now.format("%Y-%m-%d").to_string();
        let date_path = self.output_path.join(&date_folder);
        std::fs::create_dir_all(&date_path)?;

        let time_stamp = now.format("%H%M").to_string();
        let file_name = format!("sync-{}.md", time_stamp);
        let file_path = date_path.join(&file_name);

        let mut content = self.build_markdown(tasks);
        content = self.add_task_statistics(&content, tasks);

        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    fn build_markdown(&self, tasks: &[Task]) -> String {
        let mut md = String::new();

        if tasks.is_empty() {
            return md;
        }
        for task in tasks {
            self.add_task(&mut md, task)
        }
        md
    }

    fn add_task(&self, md: &mut String, task: &Task) {
        let icon = get_task_icon(task);

        let source = format!("[{}]", task.source.to_uppercase());

        md.push_str(&format!("{} {} {}\n", icon, source, task.title));

        if let Some(description) = &task.description {
            for line in description.lines() {
                md.push_str(&format!("           {}\n", line));
            }
        }

        let mut metadata: Vec<String> = Vec::new();

        let author = find_author(task);
        metadata.push(format!("by @{}", author).to_string());

        metadata.push(Task::format_absolute_time(task.created_at));

        if !metadata.is_empty() {
            md.push_str(&format!("     └─ {}\n", metadata.join(" · ")));
        }

        md.push_str(&format!("     {}\n", task.url));
    }

    fn add_task_statistics(&self, md: &str, tasks: &[Task]) -> String {
        let mut md_with_stats = md.to_string();

        let mut source_counts: HashMap<String, i64> = HashMap::new();

        for task in tasks {
            *source_counts.entry(task.source.clone()).or_insert(0) += 1;
        }

        if !source_counts.is_empty() {
            md_with_stats.push_str("\n\n## Task Statistics\n");

            for (source, count) in source_counts {
                md_with_stats.push_str(&format!("{}: {} items\n", source, count));
            }
        }

        md_with_stats
    }
}

pub fn find_author(task: &Task) -> String {
    task.people
        .iter()
        .find(|p| p.role == PersonRole::Author)
        .map(|p| format!("{}", p.username))
        .unwrap_or_else(|| "unknown".to_string())
}



pub fn get_task_icon(task: &Task) -> String {
    let icon = match task.task_type {
        TaskType::PullRequest => "🔀",
        TaskType::Issue => "🐛",
        TaskType::Review => "👀",
        TaskType::Message => "💬",
        TaskType::Ticket => "🎫",
        TaskType::Statistics => "📊",
        TaskType::MOM => "🎤",
        TaskType::Other(_) => "📌",
    };
    icon.to_string()
}
