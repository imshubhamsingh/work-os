use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CacheFileWrapper {
    pub cache: String,
}

#[derive(Debug, Deserialize)]
pub struct CacheRoot {
    pub state: CacheState,
}

#[derive(Debug, Deserialize)]
pub struct CacheState {
    #[serde(default)]
    pub documents: HashMap<String, GranolaDocument>,
    #[serde(default)]
    pub transcripts: HashMap<String, Vec<TranscriptSegment>>,
    #[serde(default)]
    #[serde(rename = "documentPanels")]
    pub document_panels: HashMap<String, HashMap<String, DocumentPanel>>,
}

#[derive(Debug, Deserialize)]
pub struct GranolaDocument {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // pub transcribe: bool,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub people: Option<PeopleInfo>,
}

impl GranolaDocument {
    // need better name
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

#[derive(Debug, Deserialize)]
pub struct PeopleInfo {
    // pub title: String,
    pub creator: Person,
    #[serde(default)]
    pub attendees: Vec<Person>,
}

#[derive(Debug, Deserialize)]
pub struct Person {
    pub email: Option<String>,
}

impl Person {
    pub fn display(&self) -> String {
        self.email.as_deref().unwrap_or("").to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptSegment {
    // pub document_id: String,
    // pub start_timestamp: String,
    // pub end_timestamp: String,
    // format: "Speaker A: <text>" - speaker embedded in text
    #[serde(default)]
    pub text: Option<String>,
    // pub source: String, // transcription service like "assemblyai" at least on desktop app, not the speaker
    // pub id: String,
    #[serde(default)]
    pub is_final: bool,
}

impl TranscriptSegment {
    pub fn display_transcripts(transcripts: Option<&[TranscriptSegment]>) -> String {
        let mut content = String::new();

        if let Some(segments) = transcripts {
            content.push_str("## Transcript\n\n");
            for segment in segments.iter().filter(|s| s.is_final) {
                let (speaker, text) = segment.get_speaker_and_text();
                content.push_str(&format!("**{}**: {}\n\n", speaker, text));
            }
        } else {
            content.push_str("## Transcript\n\n");
            content.push_str("_No transcript available for this meeting._\n\n");
        }

        content
    }

    fn get_speaker_and_text(&self) -> (&str, &str) {
        if let Some(text) = &self.text {
            if let Some((speaker, content)) = text.split_once(':') {
                let speaker = speaker.trim();
                let content = content.trim();

                if speaker.starts_with("Speaker ") || speaker.contains("Speaker") {
                    return (speaker, content);
                }
            }
            ("Unknown Speaker", text.trim())
        } else {
            ("Unknown Speaker", "")
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentPanel {
    // pub id: String,
    // pub document_id: String,
    // pub title: String,
    #[serde(default)]
    pub original_content: Option<String>, // html format summary from Granola AI
    // pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}
