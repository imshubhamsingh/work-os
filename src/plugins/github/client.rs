use crate::core::task::{
    GitHubMetadata, Person, PersonRole, Priority, ReviewCounts, Task, TaskMetadata, TaskType,
};
use crate::error::{Result, WorkOsError};
use crate::models::config::GitHubConfig;
use crate::plugins::github::model::*;
use chrono::{DateTime, Duration, Utc};
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

        let cutoff = last_24_hr_cutoff();

        for pr in prs {
            let Some((owner, repo)) = Self::parse_repo_from_url(pr.html_url.as_str()) else {
                continue;
            };

            let pr_details = self
                .get_pr_details(&owner, &repo, pr.number, cutoff)
                .await?;

            let description = Self::build_pr_description(&pr_details);
            let review_state = Self::determine_review_state(&pr_details.reviews);
            let review_counts = Self::build_review_counts(&pr_details.reviews);

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
            .with_metadata(TaskMetadata::GitHub(GitHubMetadata {
                repo: format!("{repo}"),
                number: pr.number,
                state: serde_json::to_string(&pr.state).unwrap_or_default(),
                comments: pr.comments,
                review_state,
                review_counts: Some(review_counts),
            }))
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

    fn determine_review_state(reviews: &Vec<PrReview>) -> Option<String> {
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

    fn build_review_counts(reviews: &Vec<PrReview>) -> ReviewCounts {
        ReviewCounts {
            approved: reviews
                .iter()
                .filter(|r| r.state == ReviewState::Approved)
                .count() as u32,
            changes_requested: reviews
                .iter()
                .filter(|r| r.state == ReviewState::ChangesRequested)
                .count() as u32,
            commented: reviews
                .iter()
                .filter(|r| r.state == ReviewState::Commented)
                .count() as u32,
        }
    }

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

    async fn get_pr_details(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        cutoff: DateTime<Utc>,
    ) -> Result<PrDetails> {
        let (reviews, comments, review_comments) = tokio::join!(
            self.get_reviews(owner, repo, pr_number, cutoff),
            self.get_comments(owner, repo, pr_number, cutoff),
            self.get_review_comments(owner, repo, pr_number, cutoff),
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
        cutoff: DateTime<Utc>,
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
                if submitted_at < cutoff {
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
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<PrComment>> {
        println!(
            "API call to Github: comments/{}/{}/{}/{}",
            owner, repo, pr_number, cutoff
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
                if c.created_at < cutoff {
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
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<ReviewComment>> {
        println!(
            "API call to Github: review_comments/{}/{}/{}/{}",
            owner, repo, pr_number, cutoff
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
                if c.created_at < cutoff {
                    return None;
                }
                if self.is_bot(c.user.as_ref()) {
                    println!("Review comments by Vulcahno");
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
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}

fn last_24_hr_cutoff() -> DateTime<Utc> {
    Utc::now() - Duration::hours(24)
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
