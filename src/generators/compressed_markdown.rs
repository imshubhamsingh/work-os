use chrono::Local;

use crate::core::task::{GitHubMetadata, SlackMetadata, Task, TaskMetadata, TaskType};
use crate::error::Result;
use crate::generators::markdown::{find_author, format_duration};
use std::path::PathBuf;

pub struct CompressedMarkdownGenerator {
    output_path: PathBuf,
}

impl CompressedMarkdownGenerator {
    pub fn new(path: PathBuf) -> Self {
        Self { output_path: path }
    }

    pub fn generate(&self, tasks: &[Task]) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.output_path)?;

        let file_name = Local::now().format("%Y-%m-%d-%H%M").to_string();
        let file_path = self.output_path.join(format!("{}.md", &file_name));
        let content = self.build_compressed_markdown(tasks);

        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    fn build_compressed_markdown(&self, tasks: &[Task]) -> String {
        let mut out = String::new();

        for task in tasks {
            self.add_task(&mut out, task);
            out.push('\n');
        }

        out
    }

    fn add_task(&self, out: &mut String, task: &Task) {
        match &task.metadata {
            TaskMetadata::GitHub(meta) => {
                self.write_github(out, task, meta);
            }
            TaskMetadata::Slack(meta) => {
                self.write_slack(out, task, meta);
            }
            TaskMetadata::None => {
                self.write_generic(out, task);
            }
        }
    }

    fn write_github(&self, out: &mut String, task: &Task, meta: &GitHubMetadata) {
        let author = find_author(task);
        let age = format_duration(task.created_at);

        // G|repo#number|title|by:@user|age:...|u:url
        out.push_str(&format!(
            "G|{}#{}|{}|by:{}|age:{}|u:{}",
            meta.repo,
            meta.number,
            sanitize(&task.title),
            author,
            age,
            task.url
        ));
        out.push('\n')
    }

    fn write_slack(&self, out: &mut String, task: &Task, meta: &SlackMetadata) {
        let author = find_author(task);
        let age = format_duration(task.created_at);

        let scope = slack_scope(meta);
        let name = meta.name.clone();

        let message = task.description.as_deref().unwrap_or(&task.title);

        // S|CH@M|proj-experience-os|@neel|message|age:...|u:url
        out.push_str(&format!(
            "S|{}|{}|{}|{}|age:{}|u:{}",
            scope,
            name,
            author,
            sanitize(message),
            age,
            task.url
        ));
        out.push('\n');
    }

    fn write_generic(&self, out: &mut String, task: &Task) {
        let author = find_author(task);
        let age = format_duration(task.created_at);

        // X|source|type|title|by:@user|age:...|u:url
        out.push_str(&format!(
            "X|{}|{}|{}|by:{}|age:{}|u:{}",
            task.source,
            task.task_type.short_name(),
            sanitize(&task.title),
            author,
            age,
            task.url
        ));
        out.push('\n');
    }
}

fn slack_scope(meta: &SlackMetadata) -> String {
    let base = if meta.is_dm {
        "DM"
    } else if meta.is_mpim {
        "GM"
    } else if meta.is_user_group {
        "UG"
    } else if meta.is_channel {
        "CH"
    } else {
        "" // search result
    };

    if meta.is_mention {
        format!("{}@M", base)
    } else {
        base.to_string()
    }
}

fn sanitize(input: &str) -> String {
    input.replace('\n', "\\n")
}
