use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CoralogixConfig {
    pub api_key: String,
    pub domain: String,
    pub application_names: Vec<String>,
    pub output_path: PathBuf,
}
