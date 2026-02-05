use crate::core::task::{Person, PersonRole, Priority, Task, TaskType};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;
use crate::plugins::github::model::*;
use octocrab::Octocrab;
use std::fmt::Write;

pub struct GithubClient {
    octocrab: Octocrab,
    username: String,
    include_orgs: Vec<String>,
    include_repos: Vec<String>,
    bots: Vec<String>,
}

impl GithubClient {
    pub fn new(config: &GitHubConfig) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(config.token.clone())
            .build()
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        Ok(Self {
            octocrab,
            username: config.username.clone(),
            include_orgs: config.include_orgs.clone(),
            include_repos: config.include_repos.clone(),
            bots: config.bots.clone(),
        })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let user = self
            .octocrab
            .current()
            .user()
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        Ok(user.login == self.username)
    }

    // ============================
    // Tasks
    // ============================

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let mut all_tasks = Vec::new();

        all_tasks.extend(self.get_involved_prs().await?);
        all_tasks.extend(self.get_authored_prs().await?);

        let mut all_tasks = Self::deduplicate_tasks(all_tasks);

        // Add AI usage statistics task
        match self.generate_ai_stats().await {
            Ok(stats_task) => all_tasks.push(stats_task),
            Err(e) => eprintln!("Warning: Failed to generate AI stats: {}", e),
        }

        all_tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(all_tasks)
    }

    async fn get_involved_prs(&self) -> Result<Vec<Task>> {
        let query = self.build_search_query(SearchType::Involved);
        self.get_prs(&query).await
    }

    async fn get_authored_prs(&self) -> Result<Vec<Task>> {
        let query = self.build_search_query(SearchType::Author);
        self.get_prs(&query).await
    }

    async fn get_prs(&self, query: &str) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        let prs = self.get_pr_list(query).await?;

        for pr in prs {
            let Some((owner, repo)) = Self::parse_repo_from_url(pr.html_url.as_str()) else {
                continue;
            };

            let pr_details = self.get_pr_details(&owner, &repo, pr.number).await?;

            let description = Self::build_pr_description(&pr_details);

            let mut task = Task::new(
                "github",
                TaskType::PullRequest,
                &pr.number.to_string(),
                pr.title.clone(),
                pr.html_url.to_string(),
            )
            .with_date(pr.created_at, pr.updated_at)
            .with_person(Person {
                name: pr.user.login.clone(),
                username: pr.user.login.clone(),
                role: PersonRole::Author,
            })
            .with_priority(Priority::Unknown);

            if !description.is_empty() {
                task = task.with_description(description);
            }

            tasks.push(task);
        }

        Ok(tasks)
    }

    // ============================
    // Shared helpers
    // ============================
    fn build_search_query(&self, search_type: SearchType) -> String {
        let repo_filter = self.build_repo_filter();

        if repo_filter.is_empty() {
            format!("is:pr is:open {}:{}", search_type, self.username)
        } else {
            format!(
                "is:pr is:open {}:{} {}",
                search_type, self.username, repo_filter
            )
        }
    }

    /// Build search query without repo filtering (for AI stats that should cover all repos)
    fn build_search_query_no_filter(&self, search_type: SearchType) -> String {
        format!("is:pr is:open {}:{}", search_type, self.username)
    }

    fn build_repo_filter(&self) -> String {
        if !self.include_repos.is_empty() {
            return self
                .include_repos
                .iter()
                .map(|repo| format!("repo:{}", repo))
                .collect::<Vec<_>>()
                .join(" ");
        }

        if !self.include_orgs.is_empty() {
            return self
                .include_orgs
                .iter()
                .map(|org| format!("org:{}", org))
                .collect::<Vec<_>>()
                .join(" ");
        }

        String::new()
    }

    fn parse_repo_from_url(url: &str) -> Option<(String, String)> {
        // https://github.com/owner/repo/pull/123
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 5 {
            Some((parts[3].to_string(), parts[4].to_string()))
        } else {
            None
        }
    }

    fn determine_review_state(reviews: &[PrReview]) -> Option<String> {
        /*
         * Priority: ChangesRequested > Approved > Commented
         */
        let mut has_approved = false;
        let mut has_commented = false;

        for review in reviews {
            match review.state {
                ReviewState::ChangesRequested => return Some("changes_requested".to_string()),
                ReviewState::Approved => has_approved = true,
                ReviewState::Commented => has_commented = true,
                _ => {}
            }
        }

        if has_approved {
            Some("approved".to_string())
        } else if has_commented {
            Some("commented".to_string())
        } else {
            None
        }
    }

    // fn build_review_counts(reviews: &[PrReview]) -> ReviewCounts {
    //     ReviewCounts {
    //         approved: reviews
    //             .iter()
    //             .filter(|r| r.state == ReviewState::Approved)
    //             .count() as u32,
    //         changes_requested: reviews
    //             .iter()
    //             .filter(|r| r.state == ReviewState::ChangesRequested)
    //             .count() as u32,
    //         commented: reviews
    //             .iter()
    //             .filter(|r| r.state == ReviewState::Commented)
    //             .count() as u32,
    //     }
    // }

    fn build_pr_description(details: &PrDetails) -> String {
        let mut desc = String::new();

        if !details.reviews.is_empty() {
            let _ = writeln!(desc, "Reviews:");

            for review in &details.reviews {
                let _ = writeln!(desc, "- {}:{}", review.author, review.state);

                if let Some(body) = review.body.as_ref().filter(|b| !b.is_empty()) {
                    let truncated = truncate_text(body, 300);
                    let _ = writeln!(desc, "  {}", truncated);
                }
            }

            let _ = writeln!(desc);
        }

        if !details.review_comments.is_empty() {
            let _ = writeln!(desc, "Review Comments:");

            for comment in &details.review_comments {
                let body = comment.body.trim();
                if body.is_empty() {
                    continue;
                }

                let truncated = truncate_text(body, 300);

                let location = format!(" ({})", comment.path);

                let _ = writeln!(desc, "- {}{}:", comment.author, location);
                let _ = writeln!(desc, "  {}", truncated);
            }
        }

        if !details.comments.is_empty() {
            let _ = writeln!(desc, "Comments:");

            for comment in &details.comments {
                let body = comment.body.trim();
                if body.is_empty() {
                    continue;
                }

                let truncated = truncate_text(body, 300);
                let _ = writeln!(desc, "- {}:", comment.author);
                let _ = writeln!(desc, "   {}", truncated);
            }

            let _ = writeln!(desc);
        }

        let review_state = Self::determine_review_state(&details.reviews);
        // let review_counts = Self::build_review_counts(&details.reviews);

        if let Some(review_state) = review_state {
            let _ = writeln!(desc, "Review current state: {}", review_state);
        }

        desc.trim_end().to_string()
    }

    fn deduplicate_tasks(tasks: Vec<Task>) -> Vec<Task> {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        tasks
            .into_iter()
            .filter(|task| seen.insert(task.id.clone()))
            .collect()
    }

    // ============================
    // Github API
    // ============================

    async fn get_pr_list(&self, query: &str) -> Result<Vec<octocrab::models::issues::Issue>> {
        println!("API call to Github: {}", query);
        let list = self
            .octocrab
            .search()
            .issues_and_pull_requests(query)
            .send()
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;
        Ok(list.items)
    }

    async fn get_pr_details(&self, owner: &str, repo: &str, pr_number: u64) -> Result<PrDetails> {
        let date_range = DateRange::get();

        let (reviews, comments, review_comments) = tokio::join!(
            self.get_reviews(owner, repo, pr_number, date_range),
            self.get_comments(owner, repo, pr_number, date_range),
            self.get_review_comments(owner, repo, pr_number, date_range),
        );

        Ok(PrDetails {
            body: None,
            reviews: reviews.unwrap_or_default(),
            comments: comments.unwrap_or_default(),
            review_comments: review_comments.unwrap_or_default(),
        })
    }

    async fn get_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        date_range: &DateRange,
    ) -> Result<Vec<PrReview>> {
        println!(
            "API call to Github: reviews/{}/{}/{}",
            owner, repo, pr_number
        );
        let raw_reviews = self
            .octocrab
            .pulls(owner, repo)
            .list_reviews(pr_number)
            .send()
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        let reviews = raw_reviews
            .items
            .into_iter()
            .filter_map(|r| {
                let submitted_at = r.submitted_at?;
                if !date_range.contains(submitted_at) {
                    return None;
                }

                if self.is_bot(r.user.as_ref()) {
                    return None;
                }

                let state_str = format!("{:?}", r.state?);
                Some(PrReview {
                    author: r.user?.login,
                    state: ReviewState::from_str(&state_str),
                    submitted_at,
                    body: r.body,
                })
            })
            .collect();

        Ok(reviews)
    }

    async fn get_comments(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        date_range: &DateRange,
    ) -> Result<Vec<PrComment>> {
        println!(
            "API call to Github: comments/{}/{}/{}/{}",
            owner, repo, pr_number, date_range
        );
        let raw_comments = self
            .octocrab
            .issues(owner, repo)
            .list_comments(pr_number)
            .send()
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        let comments = raw_comments
            .items
            .into_iter()
            .filter_map(|c| {
                if !date_range.contains(c.created_at) {
                    return None;
                }

                if self.is_bot(Some(&c.user)) {
                    return None;
                }

                Some(PrComment {
                    author: c.user.login,
                    body: c.body.unwrap_or_default(),
                    created_at: c.created_at,
                })
            })
            .collect();

        Ok(comments)
    }

    async fn get_review_comments(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        date_range: &DateRange,
    ) -> Result<Vec<ReviewComment>> {
        println!(
            "API call to Github: review_comments/{}/{}/{}/{}",
            owner, repo, pr_number, date_range
        );
        let raw_review_comments = self
            .octocrab
            .pulls(owner, repo)
            .list_comments(Some(pr_number))
            .send()
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        let review_comments = raw_review_comments
            .items
            .into_iter()
            .filter_map(|c| {
                if !date_range.contains(c.created_at) {
                    return None;
                }
                if self.is_bot(c.user.as_ref()) {
                    return None;
                }
                Some(ReviewComment {
                    author: c.user?.login,
                    body: c.body,
                    path: c.path,
                    created_at: c.created_at,
                })
            })
            .collect();

        Ok(review_comments)
    }

    fn is_bot(&self, user: Option<&octocrab::models::Author>) -> bool {
        user.as_ref()
            .map(|u| self.bots.contains(&u.login))
            .unwrap_or(false)
    }
}

// ============================
// Utilities
// ============================

fn truncate_text(text: &str, max_len: usize) -> String {
    let text = text.trim();
    let chars = text.chars();
    if chars.clone().count() <= max_len {
        return text.to_string();
    }
    let truncated: String = chars.take(max_len).collect();
    format!("{}...", truncated)
}

    // ============================
    // AI Usage Tracking
    // ============================

impl GithubClient {
    /// Generate AI usage statistics for merged/updated PRs in the date range
    pub async fn generate_ai_stats(&self) -> Result<Task> {
        use crate::plugins::github::ai_stats::{AIUsageStats, PrAIStats};
        use crate::plugins::github::commit_analyzer::CommitMessageAnalyzer;

        let date_range = DateRange::get();
        let stats_date = date_range.end.date_naive();
        let mut stats = AIUsageStats::new(stats_date);

        // Get all PRs from all repos (not filtered by include_repos config)
        let all_prs = self.get_all_pr_tasks_unfiltered().await?;

        let analyzer = CommitMessageAnalyzer::new();

        for task in all_prs {
            // Filter by date range - only include PRs updated within the range
            if task.updated_at < date_range.start || task.updated_at > date_range.end {
                continue;
            }
            // Extract PR info from task ID (format: "github:pr:owner/repo#number")
            let pr_info = match Self::parse_pr_id(&task.id) {
                Some(info) => info,
                None => continue,
            };

            // Fetch PR details to get LOC stats
            let pr = match self
                .octocrab
                .pulls(&pr_info.owner, &pr_info.repo)
                .get(pr_info.number)
                .await
            {
                Ok(pr) => pr,
                Err(_) => continue,
            };

            let lines_added = pr.additions.unwrap_or(0) as u32;
            let lines_deleted = pr.deletions.unwrap_or(0) as u32;

            if lines_added == 0 {
                continue; // Skip PRs with no code changes
            }

            // Fetch commit messages
            let commits = match self.get_pr_commits(&pr_info.owner, &pr_info.repo, pr_info.number).await {
                Ok(commits) => commits,
                Err(_) => continue,
            };

            // Analyze commits for AI usage
            let analysis = analyzer.analyze_pr_commits(&commits);

            // Calculate AI vs Human LOC
            let ai_loc = (lines_added as f32 * analysis.aggregate_score).round() as u32;
            let human_loc = lines_added.saturating_sub(ai_loc);

            stats.add_pr(PrAIStats {
                pr_number: pr_info.number,
                repo: format!("{}/{}", pr_info.owner, pr_info.repo),
                title: task.title.clone(),
                lines_added,
                lines_deleted,
                ai_score: analysis.aggregate_score,
                ai_loc,
                human_loc,
                commit_count: analysis.commit_count,
                has_explicit_attribution: analysis.has_explicit_attribution,
            });
        }

        Ok(stats.to_task())
    }

    /// Get commit messages for a PR
    async fn get_pr_commits(&self, owner: &str, repo: &str, pr_number: u64) -> Result<Vec<(String, String)>> {
        use octocrab::Page;

        let route = format!("/repos/{}/{}/pulls/{}/commits", owner, repo, pr_number);
        let commits: Page<octocrab::models::repos::RepoCommit> = self
            .octocrab
            .get(&route, None::<&()>)
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        let mut result = Vec::new();
        for commit in commits.items {
            result.push((commit.sha, commit.commit.message));
        }

        Ok(result)
    }

    /// Get all PR tasks without deduplication
    async fn get_all_pr_tasks(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        tasks.extend(self.get_involved_prs().await?);
        tasks.extend(self.get_authored_prs().await?);
        Ok(tasks)
    }

    /// Get all PR tasks without repo filtering (for AI stats)
    async fn get_all_pr_tasks_unfiltered(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Get involved PRs without repo filter
        let involved_query = self.build_search_query_no_filter(SearchType::Involved);
        tasks.extend(self.get_prs(&involved_query).await?);

        // Get authored PRs without repo filter
        let authored_query = self.build_search_query_no_filter(SearchType::Author);
        tasks.extend(self.get_prs(&authored_query).await?);

        Ok(tasks)
    }

    /// Parse PR ID from task ID (format: "github:pr:owner/repo#number")
    fn parse_pr_id(task_id: &str) -> Option<PrInfo> {
        let parts: Vec<&str> = task_id.split(':').collect();
        if parts.len() < 3 || parts[0] != "github" || parts[1] != "pr" {
            return None;
        }

        let pr_part = parts[2];
        let repo_pr: Vec<&str> = pr_part.split('#').collect();
        if repo_pr.len() != 2 {
            return None;
        }

        let owner_repo: Vec<&str> = repo_pr[0].split('/').collect();
        if owner_repo.len() != 2 {
            return None;
        }

        let number = repo_pr[1].parse::<u64>().ok()?;

        Some(PrInfo {
            owner: owner_repo[0].to_string(),
            repo: owner_repo[1].to_string(),
            number,
        })
    }
}

struct PrInfo {
    owner: String,
    repo: String,
    number: u64,
}

pub enum SearchType {
    Involved,
    Author,
}

impl std::fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::Involved => write!(f, "involves"),
            SearchType::Author => write!(f, "author"),
        }
    }
}
