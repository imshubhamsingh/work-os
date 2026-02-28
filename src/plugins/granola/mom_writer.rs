use chrono::{Local, Utc};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::error::{Result, WorkOsError};
use crate::plugins::granola::model::{DocumentPanel, GranolaDocument, TranscriptSegment};

const DEFAULT_SUMMARY: &str = "*No AI summary was generated for this meeting.*\n";

pub struct MomWriter {
    output_path: PathBuf,
}

impl MomWriter {
    pub fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    pub fn write_meeting_folder(
        &self,
        doc: &GranolaDocument,
        transcript: Option<&[TranscriptSegment]>,
        panel: Option<&DocumentPanel>,
    ) -> Result<(PathBuf, String)> {
        // Create path: raw/YYYY-MM-DD/moms/meeting-name/
        let date_folder = doc
            .created_at
            .unwrap_or_else(Utc::now)
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string();
        let mom_date_folder = self.output_path.join(&date_folder).join("moms");

        let meeting_folder_name =
            self.sanitize_title(doc.title.as_deref().unwrap_or("untitled"));
        let meeting_folder = mom_date_folder.join(&meeting_folder_name);

        fs::create_dir_all(&meeting_folder)?;

        let transcript_content = self.format_transcript_file(doc, transcript);
        let transcript_path = meeting_folder.join("transcript.md");
        fs::write(&transcript_path, transcript_content)
            .map_err(|e| WorkOsError::Granola(format!("Failed to write transcript.md: {}", e)))?;

        let summary_content = self.get_summary_content(doc, panel);
        let summary_path = meeting_folder.join("summary.md");
        fs::write(&summary_path, &summary_content)
            .map_err(|e| WorkOsError::Granola(format!("Failed to write summary.md: {}", e)))?;

        Ok((meeting_folder, summary_content))
    }

    fn format_transcript_file(
        &self,
        doc: &GranolaDocument,
        transcripts: Option<&[TranscriptSegment]>,
    ) -> String {
        let mut content = String::new();
        let title = doc.title.as_deref().unwrap_or("Untitled Meeting");
        content.push_str(&format!("# {}\n\n", title));
        content.push_str(&self.get_metadata(doc));
        content.push_str(&TranscriptSegment::display_transcripts(transcripts));
        content
    }

    fn get_summary_content(&self, doc: &GranolaDocument, panel: Option<&DocumentPanel>) -> String {
        let mut content = String::new();
        content.push_str(&self.get_metadata(doc));

        let summary_markdown = panel
            .and_then(|p| p.content.as_ref())
            .map(|c| prosemirror_to_markdown(c))
            .unwrap_or_else(|| DEFAULT_SUMMARY.to_string());

        content.push_str(&summary_markdown);
        content
    }

    fn get_metadata(&self, doc: &GranolaDocument) -> String {
        let mut content = String::new();

        let title = doc.title.as_deref().unwrap_or("Untitled Meeting");
        content.push_str(&format!("# {} - Summary\n\n", title));

        let date_str = doc
            .created_at
            .unwrap_or_else(Utc::now)
            .with_timezone(&Local)
            .format("%B %e, %Y at %l:%M %p");
        content.push_str(&format!("**Date**: {}\n", date_str));

        let attendees = doc.get_attendees_formated();
        if !attendees.is_empty() {
            content.push_str("**Attendees**:\n");
            for attendee in attendees {
                content.push_str(&format!("- {}\n", attendee));
            }
        }

        content.push_str("\n---\n\n");
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

// ============================
// ProseMirror JSON → Markdown
// ============================

pub fn prosemirror_to_markdown(node: &Value) -> String {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match node_type {
        "doc" => node_children(node)
            .map(|c| {
                c.iter()
                    .map(prosemirror_to_markdown)
                    .collect::<String>()
            })
            .unwrap_or_default(),

        "paragraph" => {
            let text: String = node_children(node)
                .map(|c| c.iter().map(prosemirror_to_markdown).collect())
                .unwrap_or_default();
            if text.trim().is_empty() {
                "\n".to_string()
            } else {
                format!("{}\n\n", text.trim_end())
            }
        }

        "heading" => {
            let level = node
                .get("attrs")
                .and_then(|a| a.get("level"))
                .and_then(|l| l.as_u64())
                .unwrap_or(1) as usize;
            let text: String = node_children(node)
                .map(|c| c.iter().map(prosemirror_to_markdown).collect())
                .unwrap_or_default();
            format!("{} {}\n\n", "#".repeat(level), text.trim())
        }

        "bulletList" => {
            let items: String = node_children(node)
                .map(|c| {
                    c.iter()
                        .map(|item| format_list_item(item, "- ", 0))
                        .collect()
                })
                .unwrap_or_default();
            format!("{}\n", items)
        }

        "orderedList" => {
            let items: String = node_children(node)
                .map(|c| {
                    c.iter()
                        .enumerate()
                        .map(|(i, item)| format_list_item(item, &format!("{}. ", i + 1), 0))
                        .collect()
                })
                .unwrap_or_default();
            format!("{}\n", items)
        }

        "blockquote" => {
            let inner: String = node_children(node)
                .map(|c| c.iter().map(prosemirror_to_markdown).collect())
                .unwrap_or_default();
            inner
                .lines()
                .map(|l| format!("> {}\n", l))
                .collect()
        }

        "codeBlock" => {
            let text: String = node_children(node)
                .map(|c| c.iter().map(prosemirror_to_markdown).collect())
                .unwrap_or_default();
            format!("```\n{}\n```\n\n", text)
        }

        "hardBreak" => "\n".to_string(),

        "text" => apply_marks(
            node.get("text").and_then(|t| t.as_str()).unwrap_or(""),
            node.get("marks").and_then(|m| m.as_array()),
        ),

        _ => String::new(),
    }
}

fn format_list_item(item: &Value, prefix: &str, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    let mut result = String::new();
    let mut is_first_paragraph = true;

    if let Some(children) = node_children(item) {
        for child in children {
            let child_type = child.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match child_type {
                "paragraph" => {
                    let text: String = node_children(child)
                        .map(|c| c.iter().map(prosemirror_to_markdown).collect())
                        .unwrap_or_default();
                    if is_first_paragraph {
                        result.push_str(&format!("{}{}{}\n", indent_str, prefix, text.trim()));
                        is_first_paragraph = false;
                    } else {
                        result.push_str(&format!(
                            "{}{}\n",
                            "  ".repeat(indent + 1),
                            text.trim()
                        ));
                    }
                }
                "bulletList" => {
                    if let Some(nested_items) = node_children(child) {
                        for nested in nested_items {
                            result.push_str(&format_list_item(nested, "- ", indent + 1));
                        }
                    }
                }
                "orderedList" => {
                    if let Some(nested_items) = node_children(child) {
                        for (i, nested) in nested_items.iter().enumerate() {
                            result.push_str(&format_list_item(
                                nested,
                                &format!("{}. ", i + 1),
                                indent + 1,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    result
}

fn node_children(node: &Value) -> Option<&Vec<Value>> {
    node.get("content").and_then(|c| c.as_array())
}

fn apply_marks(text: &str, marks: Option<&Vec<Value>>) -> String {
    let Some(marks) = marks else {
        return text.to_string();
    };

    let mut result = text.to_string();
    for mark in marks {
        let mark_type = mark.get("type").and_then(|t| t.as_str()).unwrap_or("");
        result = match mark_type {
            "bold" => format!("**{}**", result),
            "italic" => format!("*{}*", result),
            "code" => format!("`{}`", result),
            "link" => {
                let href = mark
                    .get("attrs")
                    .and_then(|a| a.get("href"))
                    .and_then(|h| h.as_str())
                    .unwrap_or("#");
                format!("[{}]({})", result, href)
            }
            _ => result,
        };
    }
    result
}
