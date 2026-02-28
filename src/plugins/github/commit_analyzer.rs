use serde::{Deserialize, Serialize};

use crate::plugins::github::model::PrCommit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrCommitAnalysis {
    pub commits: Vec<CommitAnalysis>,
    pub aggregate_score: f32, // average ai score across all commits
    pub has_explicit_attribution: bool,
    pub commit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAnalysis {
    pub commit_sha: String,
    pub message: String,
    pub additions: u64,
    pub deletions: u64,
    pub ai_score: f32, // 0.0 = me, 1.0 = AI
    pub signal: AISignal,
    pub is_merge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AISignal {
    ExplicitAttribution(String),
    LargeBurst, // > 1000 LOC in one commit
    LongDescriptive,
    ShortSimple,
    MergeCommit,
}

pub struct CommitMessageAnalyzer;

impl CommitMessageAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_pr_commits(&self, commits: &[&PrCommit]) -> PrCommitAnalysis {
        let mut analyses = Vec::new();
        let mut total_ai_score = 0.0;
        let mut has_explicit = false;

        for commit in commits {
            let analysis = self.analyze(
                &commit.sha,
                &commit.message,
                commit.additions,
                commit.deletions,
            );

            if matches!(analysis.signal, AISignal::ExplicitAttribution(_)) {
                has_explicit = true;
            }

            if !analysis.is_merge {
                total_ai_score += analysis.ai_score;
            }

            analyses.push(analysis);
        }

        let non_merge_count = analyses.iter().filter(|a| !a.is_merge).count() as f32;
        let average_score = if non_merge_count > 0.0 {
            total_ai_score / non_merge_count
        } else {
            0.0
        };

        PrCommitAnalysis {
            commits: analyses,
            aggregate_score: average_score,
            has_explicit_attribution: has_explicit,
            commit_count: commits.len(),
        }
    }

    pub fn analyze(
        &self,
        sha: &str,
        message: &str,
        additions: u64,
        deletions: u64,
    ) -> CommitAnalysis {
        let message_lower = message.to_lowercase();

        let is_merge = self.is_merge_commit(message);
        if is_merge {
            return CommitAnalysis {
                commit_sha: sha.to_string(),
                message: message.to_string(),
                additions,
                deletions,
                ai_score: 0.0,
                signal: AISignal::MergeCommit,
                is_merge: true,
            };
        }

        if let Some(tool) = self.check_explicit_ai_attribution(&message_lower) {
            return CommitAnalysis {
                commit_sha: sha.to_string(),
                message: message.to_string(),
                additions,
                deletions,
                ai_score: 1.0,
                signal: AISignal::ExplicitAttribution(tool),
                is_merge: false,
            };
        }

        // Check for large burst (> 1000 LOC added at once)
        if additions > 1000 {
            return CommitAnalysis {
                commit_sha: sha.to_string(),
                message: message.to_string(),
                additions,
                deletions,
                ai_score: 0.8,
                signal: AISignal::LargeBurst,
                is_merge: false,
            };
        }

        let (ai_score, signal) = self.analyze_message_pattern(message);

        CommitAnalysis {
            commit_sha: sha.to_string(),
            message: message.to_string(),
            additions,
            deletions,
            ai_score,
            signal,
            is_merge: false,
        }
    }

    fn is_merge_commit(&self, message: &str) -> bool {
        let first_line = message.lines().next().unwrap_or("");
        let first_line_lower = first_line.to_lowercase();

        // Common merge commit patterns
        first_line_lower.contains("merge") || first_line_lower.contains("merging")
    }

    fn check_explicit_ai_attribution(&self, message_lower: &str) -> Option<String> {
        for (pattern, tool) in AI_PATTERNS {
            if message_lower.contains(pattern) {
                return Some(tool.to_string());
            }
        }

        None
    }

    fn analyze_message_pattern(&self, message: &str) -> (f32, AISignal) {
        let lines: Vec<&str> = message.lines().collect();
        let line_count = lines.len();

        let has_structured_format =
            line_count >= 3 && lines.get(1).map(|l| l.trim().is_empty()).unwrap_or(false);

        match has_structured_format {
            true => (1.0, AISignal::LongDescriptive),
            false => (0.0, AISignal::ShortSimple),
        }
    }
}

impl Default for CommitMessageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

const AI_PATTERNS: [(&str, &str); 7] = [
    ("co-authored-by: claude", "Claude"),
    ("co-authored-by: anthropic", "Claude"),
    ("co-authored-by: cursor", "Cursor"),
    ("co-authored-by: openai", "ChatGPT"),
    ("generated-by: claude", "Claude"),
    ("generated-by: ai", "AI"),
    ("claude code", "Claude Code"),
];
