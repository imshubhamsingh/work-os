use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::core::task::{Task, TaskType};
use crate::plugins::github::commit_analyzer::{AISignal, CommitAnalysis};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUsageStats {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_loc_added: u64,
    pub total_loc_deleted: u64,
    pub ai_loc: u64,
    pub human_loc: u64,
    pub ai_percentage: f32,
    pub pr_stats: Vec<PrAIStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrAIStats {
    pub pr_number: u64,
    pub repo: String,
    pub title: String,
    pub lines_added: u64,
    pub lines_deleted: u64,
    pub ai_score: f32, // normalized score 0-1
    pub ai_loc: u64,
    pub human_loc: u64,
    pub commit_count: usize,
    pub has_explicit_attribution: bool,
    pub commit_details: Vec<CommitAnalysis>,
}

impl AIUsageStats {
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
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

    pub fn to_task(&self) -> Task {
        let title = format!(
            "AI Usage: {:.0}% ({} AI / {} Human LOC)",
            self.ai_percentage, self.ai_loc, self.human_loc
        );

        let description = self.to_description();

        Task::new(
            "github",
            TaskType::Statistics,
            &format!("ai-usage-{}-to-{}", self.start_date, self.end_date),
            title,
            String::new(), // No URL for stats
        )
        .with_description(description)
    }

    fn to_description(&self) -> String {
        if self.pr_stats.is_empty() {
            return format!(
                "No PRs with code changes in date range: {} to {}",
                self.start_date, self.end_date
            );
        }

        let mut desc = format!(
            "AI Usage Statistics ({} to {})\n\n\
            Total LOC Added: {}\n\
            AI LOC: {} ({:.1}%)\n\
            Human LOC: {} ({:.1}%)\n\n\
            PR Breakdown:\n",
            self.start_date,
            self.end_date,
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

            // Show commit-level breakdown
            for commit in &pr.commit_details {
                let signal_desc = match &commit.signal {
                    AISignal::ExplicitAttribution(tool) => format!("AI: {} (explicit)", tool),
                    AISignal::LargeBurst => format!("AI: Large burst (+{} LOC)", commit.additions),
                    AISignal::LongDescriptive => "AI: Long descriptive message".to_string(),
                    AISignal::MergeCommit => "Merge commit (skipped)".to_string(),
                    AISignal::ShortSimple => "Human: Simple commit".to_string(),
                };

                let score_display = if commit.is_merge {
                    "".to_string()
                } else {
                    format!(" [{:.0}%]", commit.ai_score * 100.0)
                };

                desc.push_str(&format!(
                    "    - {}: +{} -{}{} - {}\n",
                    &commit.commit_sha[..7],
                    commit.additions,
                    commit.deletions,
                    score_display,
                    signal_desc
                ));
            }
        }

        desc
    }
}
