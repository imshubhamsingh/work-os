use crate::{
    error::{Result, WorkOsError},
    plugins::granola::model::{
        CacheFileWrapper, CacheRoot, DocumentPanel, GranolaDocument, TranscriptSegment,
    },
};
use std::fs;
use std::path::PathBuf;

pub struct CacheReader {
    cache_path: PathBuf,
}

impl CacheReader {
    pub fn new() -> Result<Self> {
        let cache_path = Self::default_cache_path()?;
        Ok(Self { cache_path })
    }

    fn default_cache_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| WorkOsError::Granola("Could not determine home directory".into()))?;

        Ok(home
            .join("Library")
            .join("Application Support")
            .join("Granola")
            .join("cache-v3.json"))
    }

    pub fn is_available(&self) -> bool {
        self.cache_path.exists() && self.cache_path.is_file()
    }

    pub fn get_documents(&self) -> Result<Vec<GranolaDocument>> {
        let cache = self.read_cache()?;

        let documents: Vec<GranolaDocument> = cache
            .state
            .documents
            .into_values()
            .filter(|doc| doc.deleted_at.is_none())
            .collect();

        Ok(documents)
    }

    pub fn get_transcript(&self, document_id: &str) -> Result<Option<Vec<TranscriptSegment>>> {
        let cache = self.read_cache()?;
        let transcript = cache.state.transcripts.get(document_id).cloned();
        Ok(transcript)
    }

    pub fn get_document_panels(&self, document_id: &str) -> Result<Option<DocumentPanel>> {
        let cache = self.read_cache()?;

        if let Some(panels_map) = cache.state.document_panels.get(document_id) {
            let panel = panels_map
                .values()
                .filter(|panel| panel.deleted_at.is_none())
                .max_by_key(|panel| panel.updated_at)
                .cloned();

            Ok(panel)
        } else {
            Ok(None)
        }
    }

    pub fn read_cache(&self) -> Result<CacheRoot> {
        if !self.is_available() {
            return Err(WorkOsError::Granola(format!(
                "Granola cache file not found at {}. Is Granola installed and running?",
                self.cache_path.display()
            )));
        }

        let contents = fs::read_to_string(&self.cache_path)
            .map_err(|e| WorkOsError::Granola(format!("Failed to read cache file: {}", e)))?;

        let wrapper: CacheFileWrapper = serde_json::from_str(&contents).map_err(|e| {
            WorkOsError::Granola(format!("Failed to parse cache file wrapper: {}", e))
        })?;

        let cache: CacheRoot = serde_json::from_str(&wrapper.cache)
            .map_err(|e| WorkOsError::Granola(format!("Failed to parse cache content: {}", e)))?;

        Ok(cache)
    }
}
