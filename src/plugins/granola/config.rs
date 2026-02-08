use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GranolaConfig {
    pub output_base: PathBuf,
    pub mom_folder_name: String,
}
