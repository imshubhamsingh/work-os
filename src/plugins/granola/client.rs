use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::core::message::{Message, MessageType};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;
use crate::plugins::granola::config::GranolaConfig;
use crate::plugins::granola::model::*;
use crate::plugins::granola::mom_writer::MomWriter;

const GRANOLA_API_V1: &str = "https://api.granola.ai/v1";
const GRANOLA_API_V2: &str = "https://api.granola.ai/v2";
const CLIENT_VERSION: &str = "7.162.5";

pub struct GranolaClient {
    http: Client,
    token: String,
    refresh_token: String,
    mom_writer: MomWriter,
}

impl GranolaClient {
    pub fn new(config: &GranolaConfig) -> Result<Self> {
        let (token, refresh_token) = Self::read_tokens()?;
        let mom_writer = MomWriter::new(config.output_path.clone());
        Ok(Self {
            http: Client::new(),
            token,
            refresh_token,
            mom_writer,
        })
    }

    pub fn is_available() -> bool {
        Self::read_tokens().is_ok()
    }

    pub async fn test_connection(&self) -> Result<bool> {
        Ok(Self::is_available())
    }

    // ============================
    // Messages
    // ============================

    pub async fn get_all_messages(&mut self) -> Result<Vec<Message>> {
        let date_range = DateRange::get();
        let documents = self
            .fetch_documents(date_range.start, date_range.end)
            .await?;
        let mut messages = Vec::new();

        for doc in documents {
            match self.process_document(doc).await {
                Ok(Some(msg)) => messages.push(msg),
                Ok(None) => {}
                Err(e) => println!("  ✗ Failed to process document: {}", e),
            }
        }

        Ok(messages)
    }

    async fn process_document(&mut self, doc: GranolaDocument) -> Result<Option<Message>> {
        let Some(doc_id) = doc.id.as_deref() else {
            return Ok(None);
        };

        let Some(doc_title) = doc.title.as_deref() else {
            println!("  ↷ Skipping untitled meeting {}", doc_id);
            return Ok(None);
        };

        let transcript = match self.fetch_transcript(doc_id).await {
            Ok(t) if !t.is_empty() => Some(t),
            Ok(_) => None,
            Err(e) => {
                println!("  ⚠ Failed to fetch transcript for '{}': {}", doc_title, e);
                None
            }
        };

        let panel = match self.fetch_panels(doc_id).await {
            Ok(p) => p,
            Err(e) => {
                println!("  ⚠ Failed to fetch panels for '{}': {}", doc_title, e);
                None
            }
        };

        let (folder_path, summary) = match self
            .mom_writer
            .write_meeting_folder(&doc, transcript.as_deref(), panel.as_ref())
        {
            Ok(result) => result,
            Err(e) => {
                println!("  ✗ Failed to write MOM folder for '{}': {}", doc_title, e);
                return Ok(None);
            }
        };

        println!("  ✓ Wrote MOM folder: {}", folder_path.display());

        Ok(Some(Self::build_message(
            doc_id,
            doc_title,
            &folder_path,
            doc.created_at.unwrap_or_else(Utc::now),
            doc.updated_at.unwrap_or_else(Utc::now),
            summary,
        )))
    }

    // ============================
    // Granola API
    // ============================

    async fn fetch_documents(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<GranolaDocument>> {
        let range_body = json!({ "start": start.to_rfc3339(), "end": end.to_rfc3339() });

        // get-document-set filters by updated_at — use it as the authoritative ID list
        let set: DocumentSetResponse = self
            .post(&format!("{}/get-document-set", GRANOLA_API_V1), &range_body)
            .await?;

        // v2/get-documents returns full documents with titles but no date filter
        let title_map: HashMap<String, GranolaDocument> = self
            .post::<V2DocumentsResponse>(&format!("{}/get-documents", GRANOLA_API_V2), &json!({}))
            .await
            .map(|resp| {
                resp.docs
                    .into_iter()
                    .filter_map(|d| d.id.clone().map(|id| (id, d)))
                    .collect()
            })
            .unwrap_or_else(|e| {
                println!("  ⚠ v2/get-documents failed ({}), titles unavailable", e);
                HashMap::new()
            });

        // Merge: date-filtered IDs + full document data from title map.
        // Filter by created_at so documents only recently edited (outside the range) are excluded.
        Ok(set
            .documents
            .into_iter()
            .filter_map(|(id, entry)| {
                let doc = match title_map.get(&id) {
                    Some(full) => GranolaDocument {
                        id: Some(id),
                        title: full.title.clone(),
                        created_at: full.created_at.or(entry.updated_at),
                        updated_at: full.updated_at.or(entry.updated_at),
                        deleted_at: full.deleted_at,
                        people: full.people.clone(),
                    },
                    None => GranolaDocument {
                        id: Some(id),
                        title: None,
                        created_at: entry.updated_at,
                        updated_at: entry.updated_at,
                        deleted_at: None,
                        people: None,
                    },
                };

                let created = doc.created_at?;
                (created >= start && created <= end).then_some(doc)
            })
            .collect())
    }

    async fn fetch_transcript(&mut self, document_id: &str) -> Result<Vec<TranscriptSegment>> {
        self.post(
            &format!("{}/get-document-transcript", GRANOLA_API_V1),
            &json!({ "document_id": document_id }),
        )
        .await
    }

    async fn fetch_panels(&mut self, document_id: &str) -> Result<Option<DocumentPanel>> {
        let panels: Vec<DocumentPanel> = self
            .post(
                &format!("{}/get-document-panels", GRANOLA_API_V1),
                &json!({ "document_id": document_id }),
            )
            .await?;

        Ok(panels
            .into_iter()
            .filter(|p| p.deleted_at.is_none())
            .max_by_key(|p| p.updated_at))
    }

    // ============================
    // Auth
    // ============================

    fn read_tokens() -> Result<(String, String)> {
        let path = Self::supabase_path()?;
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| WorkOsError::Granola(format!("Failed to read supabase.json: {}", e)))?;
        let data: SupabaseJson = serde_json::from_str(&contents)
            .map_err(|e| WorkOsError::Granola(format!("Failed to parse supabase.json: {}", e)))?;
        let tokens_str = data
            .workos_tokens
            .ok_or_else(|| WorkOsError::Granola("No workos_tokens in supabase.json".into()))?;
        let tokens: WorkosTokens = serde_json::from_str(&tokens_str)
            .map_err(|e| WorkOsError::Granola(format!("Failed to parse workos_tokens: {}", e)))?;
        Ok((tokens.access_token, tokens.refresh_token))
    }

    async fn refresh_token(&mut self) -> Result<()> {
        let body = serde_json::json!({ "refresh_token": self.refresh_token });
        let resp = self
            .http
            .post(&format!("{}/refresh-access-token", GRANOLA_API_V1))
            .header("X-Client-Version", CLIENT_VERSION)
            .header("X-Granola-Platform", "macOS")
            .json(&body)
            .send()
            .await
            .map_err(|e| WorkOsError::Granola(format!("Token refresh request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(WorkOsError::Granola(format!(
                "Token refresh failed with status {}",
                resp.status()
            )));
        }

        let new_tokens: WorkosTokens = resp
            .json()
            .await
            .map_err(|e| WorkOsError::Granola(format!("Failed to parse refresh response: {}", e)))?;

        self.token = new_tokens.access_token;
        self.refresh_token = new_tokens.refresh_token;

        Ok(())
    }

    fn supabase_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| WorkOsError::Granola("Could not determine home directory".into()))?;
        Ok(home
            .join("Library")
            .join("Application Support")
            .join("Granola")
            .join("supabase.json"))
    }

    // ============================
    // HTTP
    // ============================

    async fn post<T: DeserializeOwned>(&mut self, url: &str, body: &serde_json::Value) -> Result<T> {
        let resp = self.do_post(url, body).await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            println!("  ↻ Granola token expired, refreshing...");
            self.refresh_token().await?;
            let resp = self.do_post(url, body).await?;
            return Self::parse_response(url, resp).await;
        }

        Self::parse_response(url, resp).await
    }

    async fn do_post(&self, url: &str, body: &serde_json::Value) -> Result<reqwest::Response> {
        self.http
            .post(url)
            .bearer_auth(&self.token)
            .header("X-Client-Version", CLIENT_VERSION)
            .header("X-Granola-Platform", "macOS")
            .json(body)
            .send()
            .await
            .map_err(|e| WorkOsError::Granola(format!("API request failed: {}", e)))
    }

    async fn parse_response<T: DeserializeOwned>(url: &str, resp: reqwest::Response) -> Result<T> {
        if !resp.status().is_success() {
            return Err(WorkOsError::Granola(format!(
                "Granola API returned {} for {}",
                resp.status(),
                url
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| WorkOsError::Granola(format!("Failed to read response from {}: {}", url, e)))?;

        serde_json::from_slice::<T>(&bytes).map_err(|e| {
            let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
            WorkOsError::Granola(format!(
                "Failed to parse response from {}: {} | body: {}",
                url, e, preview
            ))
        })
    }

    // ============================
    // Helpers
    // ============================

    fn build_message(
        doc_id: &str,
        title: &str,
        folder_path: &PathBuf,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        summary: String,
    ) -> Message {
        Message::new(
            "granola",
            MessageType::MOM,
            doc_id,
            title.to_string(),
            format!("file://{}", folder_path.display()),
        )
        .with_date(created_at, updated_at)
        .with_description(summary)
    }
}
