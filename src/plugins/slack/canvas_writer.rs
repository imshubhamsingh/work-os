use std::fs;
use std::path::PathBuf;

use chrono::{Local, TimeZone};

use crate::error::{Result, WorkOsError};

fn html_to_markdown(html: &str) -> String {
    let markdown = htmd::convert(html).unwrap_or_else(|_| html.to_string());

    // Remove zero-width spaces and collapse multiple blank lines into one
    let mut result = String::with_capacity(markdown.len());
    let mut consecutive_blanks = 0;

    for line in markdown.lines() {
        let trimmed = line.trim().replace('\u{200b}', "");
        if trimmed.is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= 1 {
                result.push('\n');
            }
        } else {
            consecutive_blanks = 0;
            result.push_str(&trimmed);
            result.push('\n');
        }
    }

    result.trim().to_string()
}

pub struct CanvasWriter {
    output_path: PathBuf,
}

impl CanvasWriter {
    pub fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    pub fn write_canvas(
        &self,
        _canvas_id: &str,
        title: &str,
        slack_url: &str,
        date_updated: u64,
        raw_content: &str,
        is_mentioned: bool,
        comment_mentions: &[String],
    ) -> Result<Option<(PathBuf, String)>> {
        // Create path: raw/YYYY-MM-DD/canvases/canvas-title/YYYY-MM-DD-HHmm.md
        let updated_at = Local
            .timestamp_opt(date_updated as i64, 0)
            .single()
            .unwrap_or_else(Local::now);

        let date_folder = updated_at.format("%Y-%m-%d").to_string();
        let canvas_date_folder = self.output_path.join(&date_folder).join("canvases");

        let canvas_folder_name = self.sanitize_title(title);
        let canvas_folder = canvas_date_folder.join(&canvas_folder_name);

        let file_name = format!("{}.md", updated_at.format("%Y-%m-%d-%H%M"));
        let canvas_path = canvas_folder.join(&file_name);

        let content = self.format_canvas_file(title, slack_url, date_updated, raw_content, is_mentioned, comment_mentions);

        fs::create_dir_all(&canvas_folder)?;
        fs::write(&canvas_path, &content)
            .map_err(|e| WorkOsError::Slack(format!("Failed to write canvas file: {}", e)))?;

        Ok(Some((canvas_path, content)))
    }

    fn format_canvas_file(
        &self,
        title: &str,
        slack_url: &str,
        date_updated: u64,
        raw_content: &str,
        is_mentioned: bool,
        comment_mentions: &[String],
    ) -> String {
        let mut content = String::new();

        let updated_at = Local
            .timestamp_opt(date_updated as i64, 0)
            .single()
            .unwrap_or_else(Local::now);

        content.push_str(&format!("# {}\n\n", title));
        content.push_str(&format!("**Slack URL**: {}\n", slack_url));
        content.push_str(&format!(
            "**Updated**: {}\n",
            updated_at.format("%B %e, %Y at %l:%M %p")
        ));
        if is_mentioned {
            content.push_str("**Note**: You are mentioned in the canvas content.\n");
        }
        if !comment_mentions.is_empty() {
            content.push_str("\n## Comments mentioning you\n\n");
            for comment in comment_mentions {
                content.push_str(&format!("> {}\n\n", comment));
            }
        }
        content.push_str("\n---\n\n");
        content.push_str(&html_to_markdown(raw_content));

        content
    }

    fn sanitize_title(&self, title: &str) -> String {
        title
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                ' ' => '-',
                _ => '_',
            })
            .collect::<String>()
            .trim_matches(|c| c == '-' || c == '_')
            .to_string()
    }
}
