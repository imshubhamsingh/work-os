use crate::core::task::{
    GitHubMetadata, Person, PersonRole, Priority, Task, TaskMetadata, TaskType,
};
use crate::error::{Result, WorkOsError};
use crate::models::config::GitHubConfig;
use chrono::{DateTime, Duration, Utc};
use octocrab::Octocrab;

pub struct GithubClient {
    octocrab: Octocrab,
    username: String,
    include_orgs: Vec<String>,
    include_repos: Vec<String>,
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
        })
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let (user_involved_prs, user_author_prs) = tokio::join!(
            self.get_search_issues(SearchType::Involved),
            self.get_search_issues(SearchType::Author)
        );

        let mut all_tasks = Vec::new();

        if let Ok(user_involved_prs) = user_involved_prs {
            all_tasks.extend(user_involved_prs);
        }

        if let Ok(user_author_prs) = user_author_prs {
            all_tasks.extend(user_author_prs);
        }

        all_tasks = self.deduplicate_tasks(all_tasks);

        all_tasks.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        Ok(all_tasks)
    }

    fn build_repo_filter(&self) -> String {
        if !self.include_repos.is_empty() {
            return self
                .include_repos
                .iter()
                .map(|repo| format!("repo:{}", repo))
                .collect::<Vec<String>>()
                .join(" ");
        }

        if !self.include_orgs.is_empty() {
            return self
                .include_orgs
                .iter()
                .map(|org| format!("org:{}", org))
                .collect::<Vec<String>>()
                .join(" ");
        }

        String::new()
    }

    async fn get_search_issues(&self, search_type: SearchType) -> Result<Vec<Task>> {
        let repo_filter = self.build_repo_filter();

        let query = if repo_filter.is_empty() {
            format!("is:pr is:open {}:{}", search_type.to_string(), self.username)
        } else {
            format!(
                "is:pr is:open {}:{} {}",
                search_type.to_string(), self.username, repo_filter
            )
        };

        self.search_and_convert(&query).await
    }

    async fn search_and_convert(&self, query: &str) -> Result<Vec<Task>> {
        let results = self
            .octocrab
            .search()
            .issues_and_pull_requests(&query)
            .send()
            .await
            .map_err(|e| WorkOsError::GitHub(e.to_string()))?;

        let tasks = results
            .items
            .into_iter()
            .map(|item| {
                let repo = item
                    .html_url
                    .path_segments()
                    .and_then(|mut s| {
                        let owner = s.next()?;
                        let repo = s.next()?;
                        Some(format!("{}/{}", owner, repo))
                    })
                    .unwrap_or_default();

                let created_at = item.created_at;
                let updated_at = item.updated_at;

                let priority = determine_priority(self, created_at, updated_at, &item.user.login);

                Task::new(
                    "github",
                    TaskType::PullRequest,
                    &item.number.to_string(),
                    item.title.clone(),
                    item.html_url.to_string(),
                )
                .with_date(created_at, updated_at)
                .with_person(Person {
                    name: item.user.login.clone(),
                    username: item.user.login.clone(),
                    role: PersonRole::Author,
                })
                .with_metadata(TaskMetadata::GitHub(GitHubMetadata {
                    repo,
                    number: item.number,
                    state: serde_json::to_string(&item.state).unwrap_or_default(),
                    comments: item.comments,
                    review_state: None,
                }))
                .with_priority(priority)
            })
            .collect();

        Ok(tasks)
    }

    fn deduplicate_tasks(&self, tasks: Vec<Task>) -> Vec<Task> {
        use std::collections::HashSet;
        let mut seen_task = HashSet::new();
        tasks
            .into_iter()
            .filter(|task| seen_task.insert(task.id.clone()))
            .collect()
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
}

const DAYS_TO_CRITICAL: i64 = 5;

fn determine_priority(
    client: &GithubClient,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    author: &str,
) -> Priority {
    if updated_at.signed_duration_since(created_at) > Duration::days(DAYS_TO_CRITICAL) {
        return Priority::Critical;
    }

    if author == client.username {
        return Priority::High;
    }

    Priority::Medium
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
