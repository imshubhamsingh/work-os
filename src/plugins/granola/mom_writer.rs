use crate::error::{Result, WorkOsError};
use crate::plugins::granola::model::{DocumentPanel, GranolaDocument, TranscriptSegment};
use std::fs;
use std::path::PathBuf;

const DEFAULT_SUMMARY: &str = "No summary provided by Granola";

pub struct MomWriter {
    output_base: PathBuf,
    mom_folder_name: String,
}

impl MomWriter {
    pub fn new(output_base: PathBuf, mom_folder_name: String) -> Self {
        Self {
            output_base,
            mom_folder_name,
        }
    }

    pub fn write_meeting_folder(
        &self,
        doc: &GranolaDocument,
        transcript: Option<&[TranscriptSegment]>,
        panel: Option<&DocumentPanel>,
    ) -> Result<(PathBuf, String)> {
        let date_folder = doc.created_at.format("%Y-%m-%d").to_string();
        let mom_date_folder = self
            .output_base
            .join(&self.mom_folder_name)
            .join(&date_folder);

        let meeting_folder_name = self.sanitize_title(doc.title.as_deref().unwrap_or("untitled"));
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

        let granola_summary = panel
            .and_then(|p| p.original_content.clone())
            .unwrap_or_else(|| DEFAULT_SUMMARY.to_string());

        let content = self.html_to_markdown(&granola_summary);

        content
    }

    fn get_metadata(&self, doc: &GranolaDocument) -> String {
        let mut content = String::new();

        let title = doc.title.as_deref().unwrap_or("Untitled Meeting");
        content.push_str(&format!("# {} - Summary\n\n", title));

        let date_str = doc.created_at.format("%B %e, %Y at %l:%M %p");
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

    pub fn html_to_markdown(&self, html: &str) -> String {
        let html = html
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");

        // to replace <li><p>content</p></li> with <li>content</li>
        let html = html
            .replace("<li><p>", "<li>")
            .replace("</p></li>", "</li>")
            .replace("<li> <p>", "<li>")
            .replace("</p> </li>", "</li>");

        let result = html
            // header
            .replace("<h1>", "# ")
            .replace("</h1>", "\n")
            .replace("<h2>", "## ")
            .replace("</h2>", "\n")
            .replace("<h3>", "### ")
            .replace("</h3>", "\n")
            .replace("<h4>", "#### ")
            .replace("</h4>", "\n")
            // paragraphs
            .replace("<p>", "")
            .replace("</p>", "\n\n")
            // line breaks
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            // strong/bold
            .replace("<strong>", "**")
            .replace("</strong>", "**")
            .replace("<b>", "**")
            .replace("</b>", "**")
            // itlaics
            .replace("<em>", "*")
            .replace("</em>", "*")
            .replace("<i>", "*")
            .replace("</i>", "*")
            // list
            .replace("<ul>", "")
            .replace("</ul>", "\n")
            .replace("<ol>", "")
            .replace("</ol>", "\n")
            .replace("<li>", "\n- ")
            .replace("</li>", "")
            // hr
            .replace("<hr>", "\n---\n")
            .replace("<hr/>", "\n---\n")
            .replace("<hr />", "\n---\n")
            // link
            .replace("<a href=\"", "[")
            .replace("\">", "](")
            .replace("</a>", ")")
            .trim()
            .to_string();

        let lines: Vec<&str> = result.lines().collect();
        let mut cleaned_lines = Vec::new();
        let mut prev_empty = false;
        let mut in_list = false;

        for line in lines {
            let trimmed = line.trim();
            let is_empty = trimmed.is_empty();
            let is_list_item = trimmed.starts_with("- ") || trimmed.starts_with("* ");

            if is_list_item {
                in_list = true;
            } else if !is_empty {
                in_list = false;
            }

            let should_insert_blank = !in_list && !prev_empty;

            if is_empty {
                if should_insert_blank {
                    cleaned_lines.push("");
                }
                prev_empty = true;
                continue;
            }

            cleaned_lines.push(trimmed);
            prev_empty = false;
        }

        cleaned_lines.join("\n").trim().to_string()
    }
}
