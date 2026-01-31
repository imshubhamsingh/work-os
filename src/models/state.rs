use std::path::PathBuf;

use crate::error::{Result, WorkOsError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::date_range::RunMode;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyBriefState {
    pub last_run: Option<DateTime<Utc>>,
    pub last_run_mode: Option<RunMode>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkOsState {
    pub daily_brief: DailyBriefState,
}

impl WorkOsState {
    pub fn state_path() -> Result<PathBuf> {
        let state = dirs::home_dir()
            .ok_or_else(|| WorkOsError::Config("Could not determine home directory".into()))?;
        Ok(state.join(".work-os").join("logs").join("state.json"))
    }
    pub fn load() -> Result<Self> {
        let path = Self::state_path()?;
        if !path.exists() {
            return Err(WorkOsError::State(
                "State file not found. Run: work-os config init".to_string(),
            ));
        }
        let contents = std::fs::read_to_string(&path)?;
        let state: WorkOsState = serde_json::from_str(&contents)
            .map_err(|e| WorkOsError::State(format!("Failed to parse state: {}", e)))?;

        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let state_path = Self::state_path()?;
        if let Some(state_dir) = state_path.parent() {
            std::fs::create_dir_all(state_dir)?;
        }
        let contents =
            serde_json::to_string_pretty(self).map_err(|e| WorkOsError::Config(e.to_string()))?;
        std::fs::write(&state_path, contents)?;

        Ok(())
    }

    pub fn update_daily_brief(&mut self, mode: &RunMode) -> Result<()> {
        self.daily_brief.last_run = Some(Utc::now());
        self.daily_brief.last_run_mode = Some(mode.clone());
        self.save()?;
        Ok(())
    }
}
