use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAnalysis {
    pub commit_sha: String,
    pub message: String,
    pub ai_score: f32,  // 0.0 = definitely human, 1.0 = definitely AI
    pub signal: AISignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AISignal {
    ExplicitAttribution(String),  // Co-Authored-By: Claude
    LongDescriptive,              // Long message with title + body
    ShortSimple,                  // Short, simple message
}

pub struct CommitMessageAnalyzer;

impl CommitMessageAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a commit message and determine if it's AI-generated
    pub fn analyze(&self, sha: &str, message: &str) -> CommitAnalysis {
        let message_lower = message.to_lowercase();

        // Check for explicit AI attribution (100% confidence)
        if let Some(tool) = self.check_explicit_attribution(&message_lower) {
            return CommitAnalysis {
                commit_sha: sha.to_string(),
                message: message.to_string(),
                ai_score: 1.0,
                signal: AISignal::ExplicitAttribution(tool),
            };
        }

        // Analyze message structure and length
        let (ai_score, signal) = self.analyze_message_pattern(message);

        CommitAnalysis {
            commit_sha: sha.to_string(),
            message: message.to_string(),
            ai_score,
            signal,
        }
    }

    /// Check for explicit AI tool attribution in commit message
    fn check_explicit_attribution(&self, message_lower: &str) -> Option<String> {
        let patterns = [
            ("co-authored-by: claude", "Claude"),
            ("co-authored-by: anthropic", "Claude"),
            ("co-authored-by: github copilot", "GitHub Copilot"),
            ("co-authored-by: copilot", "GitHub Copilot"),
            ("co-authored-by: cursor", "Cursor"),
            ("co-authored-by: openai", "ChatGPT"),
            ("generated-by: claude", "Claude"),
            ("generated-by: ai", "AI"),
            ("claude code", "Claude Code"),
        ];

        for (pattern, tool) in patterns {
            if message_lower.contains(pattern) {
                return Some(tool.to_string());
            }
        }

        None
    }

    /// Analyze message pattern (length, structure) to detect AI
    fn analyze_message_pattern(&self, message: &str) -> (f32, AISignal) {
        let lines: Vec<&str> = message.lines().collect();
        let total_chars = message.len();
        let line_count = lines.len();

        // Has title + body structure (blank line separating title and body)
        let has_structured_format = line_count >= 3
            && lines.get(1).map(|l| l.trim().is_empty()).unwrap_or(false);

        // Check message length thresholds
        let is_very_long = total_chars > 200;
        let is_long = total_chars > 100;
        let is_short = total_chars < 50;

        // Scoring logic based on your pattern
        if is_very_long && has_structured_format {
            // Very long with title + body = very likely AI
            (0.9, AISignal::LongDescriptive)
        } else if is_long && has_structured_format {
            // Long with title + body = likely AI
            (0.75, AISignal::LongDescriptive)
        } else if is_long && line_count >= 3 {
            // Long multi-line but no blank separator = probably AI
            (0.6, AISignal::LongDescriptive)
        } else if is_short && line_count == 1 {
            // Short single-line = likely human
            (0.1, AISignal::ShortSimple)
        } else {
            // Medium length = uncertain
            (0.4, AISignal::ShortSimple)
        }
    }

    /// Analyze all commits in a PR and calculate aggregate AI score
    pub fn analyze_pr_commits(&self, commits: &[(String, String)]) -> PrCommitAnalysis {
        let mut analyses = Vec::new();
        let mut total_ai_score = 0.0;
        let mut has_explicit = false;

        for (sha, message) in commits {
            let analysis = self.analyze(sha, message);

            if matches!(analysis.signal, AISignal::ExplicitAttribution(_)) {
                has_explicit = true;
            }

            total_ai_score += analysis.ai_score;
            analyses.push(analysis);
        }

        let commit_count = commits.len() as f32;
        let average_score = if commit_count > 0.0 {
            total_ai_score / commit_count
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrCommitAnalysis {
    pub commits: Vec<CommitAnalysis>,
    pub aggregate_score: f32,  // Average AI score across all commits
    pub has_explicit_attribution: bool,
    pub commit_count: usize,
}

impl Default for CommitMessageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
