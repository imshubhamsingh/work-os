use crate::core::message::{
    GitHubMetadata, Message, MessageMetadata, MessageType, Person, PersonRole, Priority,
};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;
use crate::plugins::github::ai_stats::{AIUsageStats, PrAIStats};
use crate::plugins::github::commit_analyzer::CommitMessageAnalyzer;
use crate::plugins::github::model::*;
use chrono::{DateTime, Local, TimeDelta, Utc};
use octocrab::Octocrab;
use std::fmt::Write;

pub struct GithubClient {
    octocrab: Octocrab,
    username: String,
    include_orgs: Vec<String>,
    bots: Vec<String>,
}

impl GithubClient {
    // ============================
    // Public API
    // ============================

    pub fn new(config: &GitHubConfig) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(config.token.clone())
            .build()
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        Ok(Self {
            octocrab,
            username: config.username.clone(),
            include_orgs: config.include_orgs.clone(),
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

    pub async fn get_all_messages(&self) -> Result<Vec<Message>> {
        let all_messages = self.get_involved_prs().await?;

        let mut all_messages = Self::deduplicate_messages(all_messages);

        match self.get_ai_stats().await {
            Ok(stats_task) => all_messages.push(stats_task),
            Err(e) => println!("Warning: Failed to generate AI stats: {}", e),
        }

        all_messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(all_messages)
    }

    // ============================
    // PR Fetching
    // ============================

    async fn get_involved_prs(&self) -> Result<Vec<Message>> {
        let query = self.build_pr_query(SearchType::Involved);
        self.get_prs(&query).await
    }

    async fn get_prs(&self, query: &str) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        let prs = self.get_pr_list(query).await?;

        for pr in prs {
            let Some((owner, repo)) = Self::parse_repo_from_url(pr.html_url.as_str()) else {
                continue;
            };

            let pr_details = self
                .get_pr_details(&owner, &repo, pr.number, pr.created_at)
                .await?;

            let description = Self::build_pr_description(&pr_details);

            let mut message = Message::new(
                "github",
                MessageType::PullRequest,
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
            .with_priority(Priority::Unknown)
            .with_metadata(MessageMetadata::GitHub(GitHubMetadata {
                repo: format!("{}/{}", owner, repo),
                number: pr.number,
                state: format!("{:?}", pr.state),
                comments: pr.comments,
                additions: None,
                deletions: None,
            }));

            if !description.is_empty() {
                message = message.with_description(description);
            }

            messages.push(message);
        }

        Ok(messages)
    }

    // ============================
    // AI Stats
    // ============================

    pub async fn get_ai_stats(&self) -> Result<Message> {
        let mut date_range = DateRange::get().clone();

        let delta = date_range.end.date_naive() - date_range.start.date_naive();
        if delta.num_days() <= 0 {
            date_range.start = date_range.start - TimeDelta::days(1);
            date_range.end = date_range.end + TimeDelta::days(1);
        }

        let start_date = date_range.start.date_naive();
        let end_date = date_range.end.date_naive();
        let mut stats = AIUsageStats::new(start_date, end_date);

        let analyzer = CommitMessageAnalyzer::new();
        let all_prs_task = self.get_all_pr_messages_lite().await?;

        for message in all_prs_task {
            let MessageMetadata::GitHub(ref metadata) = message.metadata else {
                continue;
            };

            // pr created after date range
            if message.created_at > date_range.end {
                continue;
            }

            // pr created and last updated before date range
            if message.updated_at < date_range.start && message.created_at < date_range.start {
                continue;
            }

            let Ok(commits) = self.get_pr_commits(&metadata.repo, metadata.number).await else {
                continue;
            };

            let commits_in_range: Vec<_> = commits
                .iter()
                .filter(|c| c.date >= date_range.start && c.date <= date_range.end)
                .collect();

            if commits_in_range.is_empty() {
                continue;
            }

            let analysis = analyzer.analyze_pr_commits(&commits_in_range);

            let lines_added: u64 = analysis
                .commits
                .iter()
                .filter(|c| !c.is_merge)
                .map(|c| c.additions)
                .sum();
            let lines_deleted: u64 = analysis
                .commits
                .iter()
                .filter(|c| !c.is_merge)
                .map(|c| c.deletions)
                .sum();

            if lines_added == 0 {
                continue;
            }

            let ai_loc = (lines_added as f32 * analysis.aggregate_score).round() as u64;
            let human_loc = lines_added.saturating_sub(ai_loc);

            stats.add_pr(PrAIStats {
                pr_number: metadata.number,
                repo: metadata.repo.clone(),
                title: message.title.clone(),
                lines_added,
                lines_deleted,
                ai_score: analysis.aggregate_score,
                ai_loc,
                human_loc,
                commit_count: analysis.commit_count,
                has_explicit_attribution: analysis.has_explicit_attribution,
                commit_details: analysis.commits.clone(),
            });
        }

        Ok(stats.to_message())
    }

    async fn get_all_pr_messages_lite(&self) -> Result<Vec<Message>> {
        let authored_query = self.build_pr_query(SearchType::Involved);
        let prs = self.get_pr_list(&authored_query).await?;

        let messages = prs
            .into_iter()
            .filter_map(|pr| {
                let (owner, repo) = Self::parse_repo_from_url(pr.html_url.as_str())?;

                Some(
                    Message::new(
                        "github",
                        MessageType::PullRequest,
                        &pr.number.to_string(),
                        pr.title.clone(),
                        pr.html_url.to_string(),
                    )
                    .with_date(pr.created_at, pr.updated_at)
                    .with_metadata(MessageMetadata::GitHub(GitHubMetadata {
                        repo: format!("{}/{}", owner, repo),
                        number: pr.number,
                        state: format!("{:?}", pr.state),
                        comments: pr.comments,
                        additions: None,
                        deletions: None,
                    })),
                )
            })
            .collect();

        Ok(messages)
    }

    // ============================
    // GitHub API
    // ============================

    async fn get_pr_commits(
        &self,
        repo: &str, // this will always be "owner/repo"
        pr_number: u64,
    ) -> Result<Vec<PrCommit>> {
        use octocrab::Page;

        let route = format!("/repos/{}/pulls/{}/commits?per_page=5000", repo, pr_number);
        println!("API call to Github: {}", route);

        let first_page: Page<octocrab::models::repos::RepoCommit> = self
            .octocrab
            .get(&route, None::<&()>)
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        let commits: Vec<octocrab::models::repos::RepoCommit> = self
            .octocrab
            .all_pages(first_page)
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        let mut result = Vec::new();
        for commit in commits {
            let Some(date) = commit.commit.author.and_then(|a| a.date) else {
                continue;
            };

            let commit_route = format!("/repos/{}/commits/{}", repo, commit.sha);
            let full_commit: octocrab::models::repos::RepoCommit = self
                .octocrab
                .get(&commit_route, None::<&()>)
                .await
                .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

            let additions = full_commit
                .stats
                .as_ref()
                .and_then(|s| s.additions)
                .unwrap_or(0);
            let deletions = full_commit
                .stats
                .as_ref()
                .and_then(|s| s.deletions)
                .unwrap_or(0);

            result.push(PrCommit {
                sha: commit.sha,
                message: commit.commit.message,
                date,
                additions,
                deletions,
            });
        }

        Ok(result)
    }

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

    async fn get_pr_details(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        pr_created_at: DateTime<Utc>,
    ) -> Result<PrDetails> {
        let mut date_range = DateRange::get().clone();

        let delta = date_range.end.date_naive() - date_range.start.date_naive();
        if delta.num_days() <= 0 {
            date_range.start = date_range.start - TimeDelta::days(1);
            date_range.end = date_range.end + TimeDelta::days(1);
        }

        let body = if date_range.contains(pr_created_at) {
            let pulls = self.octocrab.pulls(owner, repo);
            pulls
                .get(pr_number)
                .await
                .ok()
                .and_then(|p| p.body)
                .filter(|b| !b.trim().is_empty())
        } else {
            None
        };

        let (reviews, comments, review_comments) = tokio::join!(
            self.get_reviews(owner, repo, pr_number, &date_range),
            self.get_comments(owner, repo, pr_number, &date_range),
            self.get_review_comments(owner, repo, pr_number, &date_range),
        );

        Ok(PrDetails {
            body,
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

    // ============================
    // Builders
    // ============================

    fn build_pr_query(&self, search_type: SearchType) -> String {
        let date_range = DateRange::get();
        let since = date_range.start.format("%Y-%m-%d");

        let org_filter = if !self.include_orgs.is_empty() {
            format!(
                " {}",
                self.include_orgs
                    .iter()
                    .map(|org| format!("org:{}", org))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        } else {
            String::new()
        };

        format!(
            "is:pr {}:{} updated:>={}{}",
            search_type, self.username, since, org_filter
        )
    }

    fn build_pr_description(details: &PrDetails) -> String {
        let mut desc = String::new();

        if let Some(body) = &details.body {
            let _ = writeln!(desc, "Description:\n{}\n", body);
        }

        if !details.reviews.is_empty() {
            let _ = writeln!(desc, "Reviews:");

            for review in &details.reviews {
                let ts = review
                    .submitted_at
                    .with_timezone(&Local)
                    .format("%b %d, %l:%M %p");
                let _ = writeln!(desc, "- {} [{}]: {}", review.author, ts, review.state);

                if let Some(truncated) = review.truncated_body(300).filter(|b| !b.is_empty()) {
                    let _ = writeln!(desc, "  {}", truncated);
                }
            }

            let _ = writeln!(desc);
        }

        if !details.review_comments.is_empty() {
            let _ = writeln!(desc, "Review Comments:");

            for comment in &details.review_comments {
                let truncated = comment.truncated_body(300);
                if truncated.trim().is_empty() {
                    continue;
                }

                let ts = comment
                    .created_at
                    .with_timezone(&Local)
                    .format("%b %d, %l:%M %p");
                let location = format!(" ({})", comment.path);
                let _ = writeln!(desc, "- {} [{}]{}:", comment.author, ts, location);
                let _ = writeln!(desc, "  {}", truncated);
            }
        }

        if !details.comments.is_empty() {
            let _ = writeln!(desc, "Comments:");

            for comment in &details.comments {
                let truncated = comment.truncated_body(300);
                if truncated.trim().is_empty() {
                    continue;
                }

                let ts = comment
                    .created_at
                    .with_timezone(&Local)
                    .format("%b %d, %l:%M %p");
                let _ = writeln!(desc, "- {} [{}]:", comment.author, ts);
                let _ = writeln!(desc, "   {}", truncated);
            }

            let _ = writeln!(desc);
        }

        let review_state = Self::determine_review_state(&details.reviews);

        if let Some(review_state) = review_state {
            let _ = writeln!(desc, "Review current state: {}", review_state);
        }

        desc.trim_end().to_string()
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

    // ============================
    // Helpers
    // ============================

    fn deduplicate_messages(messages: Vec<Message>) -> Vec<Message> {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        messages
            .into_iter()
            .filter(|message| seen.insert(message.id.clone()))
            .collect()
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

    fn is_bot(&self, user: Option<&octocrab::models::Author>) -> bool {
        user.as_ref()
            .map(|u| self.bots.contains(&u.login))
            .unwrap_or(false)
    }
}
