use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

// ============================
// Auth
// ============================

#[derive(Debug, Deserialize)]
pub struct SupabaseJson {
    /// Stored as a JSON-encoded string, not a nested object
    pub workos_tokens: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkosTokens {
    pub access_token: String,
}

// ============================
// Documents
// ============================

#[derive(Debug, Deserialize)]
pub struct DocumentSetResponse {
    pub documents: HashMap<String, DocumentSetEntry>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentSetEntry {
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct V2DocumentsResponse {
    pub docs: Vec<GranolaDocument>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GranolaDocument {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub people: Option<PeopleInfo>,
}

impl GranolaDocument {
    pub fn get_attendees_formated(&self) -> Vec<String> {
        let mut attendees = Vec::new();
        if let Some(people) = &self.people {
            attendees.push(people.creator.display());
            for attendee in &people.attendees {
                attendees.push(attendee.display());
            }
        }
        attendees
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PeopleInfo {
    pub creator: Person,
    #[serde(default)]
    pub attendees: Vec<Person>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Person {
    pub email: Option<String>,
}

impl Person {
    pub fn display(&self) -> String {
        self.email.as_deref().unwrap_or("").to_string()
    }
}

// ============================
// Transcript
// ============================

#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptSegment {
    #[serde(default)]
    pub text: Option<String>,
    /// "microphone" = you speaking, "system" = other participants' audio
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub is_final: bool,
}

impl TranscriptSegment {
    pub fn display_transcripts(transcripts: Option<&[TranscriptSegment]>) -> String {
        let mut content = String::new();
        content.push_str("## Transcript\n\n");

        match transcripts {
            Some(segments) => {
                for segment in segments.iter().filter(|s| s.is_final) {
                    let (speaker, text) = segment.get_speaker_and_text();
                    if !text.is_empty() {
                        content.push_str(&format!("**{}**: {}\n\n", speaker, text));
                    }
                }
            }
            None => {
                content.push_str("_No transcript available for this meeting._\n\n");
            }
        }

        content
    }

    fn get_speaker_and_text(&self) -> (&str, &str) {
        let speaker = match self.source.as_deref() {
            Some("microphone") => "You (microphone)",
            Some("system") => "Other (system audio)",
            _ => "Unknown",
        };
        let text = self.text.as_deref().unwrap_or("").trim();
        (speaker, text)
    }
}

// ============================
// Panels
// ============================

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentPanel {
    /// ProseMirror JSON document from Granola AI summary
    #[serde(default)]
    pub content: Option<Value>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}
