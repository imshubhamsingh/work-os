use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::core::task::{Task, TaskType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUsageStats {
    pub date: NaiveDate,
    pub total_loc_added: u32,
    pub total_loc_deleted: u32,
    pub ai_loc: u32,
    pub human_loc: u32,
    pub ai_percentage: f32,
    pub pr_stats: Vec<PrAIStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrAIStats {
    pub pr_number: u64,
    pub repo: String,
    pub title: String,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub ai_score: f32, // 0.0 - 1.0
    pub ai_loc: u32,
    pub human_loc: u32,
    pub commit_count: usize,
    pub has_explicit_attribution: bool,
}

impl AIUsageStats {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            total_loc_added: 0,
            total_loc_deleted: 0,
            ai_loc: 0,
            human_loc: 0,
            ai_percentage: 0.0,
            pr_stats: Vec::new(),
        }
    }

    pub fn add_pr(&mut self, pr: PrAIStats) {
        self.total_loc_added += pr.lines_added;
        self.total_loc_deleted += pr.lines_deleted;
        self.ai_loc += pr.ai_loc;
        self.human_loc += pr.human_loc;
        self.pr_stats.push(pr);
        self.ai_percentage = if self.total_loc_added > 0 {
            (self.ai_loc as f32 / self.total_loc_added as f32) * 100.0
        } else {
            0.0
        };
    }

    /// Convert to a Task for display in daily brief
    pub fn to_task(&self) -> Task {
        let title = format!(
            "AI Usage: {:.0}% ({} AI / {} Human LOC)",
            self.ai_percentage, self.ai_loc, self.human_loc
        );

        let description = self.to_description();

        Task::new(
            "github",
            TaskType::Statistics,
            &format!("ai-usage-{}", self.date),
            title,
            String::new(), // No URL for stats
        )
        .with_description(description)
    }

    /// Generate markdown description
    fn to_description(&self) -> String {
        if self.pr_stats.is_empty() {
            return "No PRs with code changes today".to_string();
        }

        let mut desc = format!(
            "Daily AI Usage - {}\n\n\
            Total LOC Added: {}\n\
            AI LOC: {} ({:.1}%)\n\
            Human LOC: {} ({:.1}%)\n\n\
            PR Breakdown:\n",
            self.date,
            self.total_loc_added,
            self.ai_loc,
            self.ai_percentage,
            self.human_loc,
            100.0 - self.ai_percentage,
        );

        for pr in &self.pr_stats {
            let attribution = if pr.has_explicit_attribution {
                " [Explicit AI]"
            } else {
                ""
            };

            desc.push_str(&format!(
                "  #{} ({}): +{} LOC, AI: {:.0}%{} → AI: {} | Human: {} | Commits: {}\n",
                pr.pr_number,
                pr.repo,
                pr.lines_added,
                pr.ai_score * 100.0,
                attribution,
                pr.ai_loc,
                pr.human_loc,
                pr.commit_count,
            ));
        }

        desc
    }

    /// Generate markdown table format
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "## Daily AI Usage - {}\n\n\
            | Metric | Value |\n\
            |--------|-------|\n\
            | Total LOC Added | {} |\n\
            | AI LOC | {} ({:.1}%) |\n\
            | Human LOC | {} ({:.1}%) |\n\n\
            ### PR Breakdown\n\n\
            | PR | Repo | LOC | AI Score | AI LOC | Human LOC | Commits |\n\
            |----|------|-----|----------|--------|-----------|---------|",
            self.date,
            self.total_loc_added,
            self.ai_loc,
            self.ai_percentage,
            self.human_loc,
            100.0 - self.ai_percentage,
        );

        for pr in &self.pr_stats {
            let attribution = if pr.has_explicit_attribution {
                " ✓"
            } else {
                ""
            };

            md.push_str(&format!(
                "\n| #{} | {} | +{} | {:.0}%{} | {} | {} | {} |",
                pr.pr_number,
                pr.repo,
                pr.lines_added,
                pr.ai_score * 100.0,
                attribution,
                pr.ai_loc,
                pr.human_loc,
                pr.commit_count,
            ));
        }

        md
    }
}
