use crate::core::task::{Task, TaskType};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;
use crate::plugins::granola::cache_reader::CacheReader;
use crate::plugins::granola::config::GranolaConfig;
use crate::plugins::granola::mom_writer::MomWriter;

pub struct GranolaClient {
    cache_reader: CacheReader,
    mom_writer: MomWriter,
}

impl GranolaClient {
    pub fn new(config: &GranolaConfig) -> Result<Self> {
        let cache_reader = CacheReader::new()?;

        if !cache_reader.is_available() {
            return Err(WorkOsError::Granola(
                       "Granola cache file not found at ~/Library/Application Support/Granola/cache-v3.json. \
                        Install Granola and create at least one meeting note.".into(),
                   ));
        }

        let mom_writer = MomWriter::new(config.output_base.clone(), config.mom_folder_name.clone());

        Ok(Self {
            cache_reader,
            mom_writer,
        })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        self.cache_reader.read_cache()?;
        Ok(true)
    }

    pub async fn get_all_tasks(&mut self) -> Result<Vec<Task>> {
        let date_range = DateRange::get();
        let documents = self.cache_reader.get_documents()?;

        let filtered_docs: Vec<_> = documents
            .into_iter()
            .filter(|doc| {
                date_range.contains(doc.created_at) || date_range.contains(doc.updated_at)
            })
            .collect();

        let mut tasks = Vec::new();

        for doc in filtered_docs {
            let doc_id = doc.id.as_deref().unwrap_or("unknown");
            let doc_title = doc.title.as_deref().unwrap_or("Untitled Meeting");

            let transcript = self.cache_reader.get_transcript(doc_id)?;
            let panel = self.cache_reader.get_document_panels(doc_id)?;

            match self
                .mom_writer
                .write_meeting_folder(&doc, transcript.as_deref(), panel.as_ref())
            {
                Ok((folder_path, granola_summary)) => {
                    println!("  ✓ Wrote MOM folder: {}", folder_path.display());
                    let task = Task::new(
                        "granola",
                        TaskType::Other("meeting".to_string()),
                        doc_id,
                        doc_title.to_string(),
                        format!("file://{}", folder_path.display()),
                    )
                    .with_date(doc.created_at, doc.updated_at)
                    .with_description(granola_summary);

                    tasks.push(task);
                }
                Err(e) => {
                    println!("  ✗ Failed to write MOM folder for '{}': {}", doc_title, e);
                }
            }
        }

        Ok(tasks)
    }
}
