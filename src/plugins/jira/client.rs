use std::collections::{HashMap, HashSet};

use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, FixedOffset, Local, Utc};
use regex::Regex;
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::core::message::{Person, PersonRole, Priority, Message, MessageType};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;
use crate::plugins::jira::config::{JiraConfig, JqlFilter};
use crate::plugins::jira::model::*;

const JIRA_API_BASE_PATH: &str = "/rest/api/3";

const MAX_ISSUES_LIMIT: usize = 100;

pub struct JiraClient {
    http: Client,
    domain: String,
    auth_header: String,
    filters: Vec<JqlFilter>,
    user_cache: HashMap<String, JiraUser>,
}

impl JiraClient {
    pub fn new(config: &JiraConfig) -> Result<Self> {
        let credentials = format!("{}:{}", config.email, config.token);
        let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
        let auth_header = format!("Basic {}", encoded);

        Ok(Self {
            http: Client::new(),
            domain: config.domain.clone(),
            auth_header,
            filters: config.filters.clone(),
            user_cache: HashMap::new(),
        })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        self.get::<JiraMyselfResponse>("myself", None).await?;
        Ok(true)
    }

    pub async fn get_all_messages(&mut self) -> Result<Vec<Message>> {
        if self.filters.is_empty() {
            println!("No Jira filters configured");
            return Ok(Vec::new());
        }

        let filters: Vec<(String, Priority, String)> = self
            .filters
            .iter()
            .filter(|f| f.enabled)
            .map(|f| {
                let range = DateRange::get();
                let date_jql = format!(
                    "updated >= \"{}\" AND updated <= \"{}\"",
                    range.start.format("%Y-%m-%d"),
                    range.end.format("%Y-%m-%d")
                );
                let jql_with_date = format!("({}) AND {}", f.jql, date_jql);

                let priority = f.priority_enum();
                let name = f.name.clone();
                (jql_with_date, priority, name)
            })
            .collect();

        let mut all_messages = Vec::new();
        let mut failed = 0usize;
        let total = filters.len();

        for (jql_with_date, priority, name) in filters {
            match self.search_issues(&jql_with_date, priority).await {
                Ok(messages) => all_messages.extend(messages),
                Err(e) => {
                    eprintln!("  Jira filter '{}' failed: {}", name, e);
                    failed += 1;
                }
            }
        }

        if failed == total {
            return Err(WorkOsError::Jira(
                "All Jira filters failed — token may be expired or domain unreachable. \
                 Run: work-os config set jira token <NEW_TOKEN>"
                    .into(),
            ));
        }

        all_messages = deduplicate_messages(all_messages);
        all_messages.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });

        Ok(all_messages)
    }

    // ============================
    // Jira API
    // ============================

    async fn search_issues(&mut self, jql: &str, default_priority: Priority) -> Result<Vec<Message>> {
        let fields = vec![
            "summary",
            "status",
            "priority",
            "assignee",
            "reporter",
            "created",
            "updated",
            "duedate",
            "labels",
            "issuetype",
            "project",
            "description",
            "components",
            "resolution",
            "comment",
            "parent",
            "customfield_10020", // sprint id
            "customfield_10014", // epic id
        ]
        .join(",");

        let data: JiraSearchResponse = self
            .get(
                "search/jql",
                Some(&[
                    ("jql", jql),
                    ("fields", &fields),
                    ("expand", "changelog"),
                    ("maxResults", &MAX_ISSUES_LIMIT.to_string()),
                ]),
            )
            .await?;

        let mut messages: Vec<Message> = data
            .issues
            .into_iter()
            .map(|issue| issue_to_message(issue, &self.domain, default_priority.clone()))
            .collect();

        // Replace user mentions in descriptions
        for message in &mut messages {
            if let Some(ref desc) = message.description {
                message.description = Some(self.replace_user_mentions(desc).await?);
            }
        }

        Ok(messages)
    }

    async fn get<T: DeserializeOwned>(
        &self,
        end_point: &str,
        query: Option<&[(&str, &str)]>,
    ) -> Result<T> {
        let url = format!(
            "https://{}{}/{}",
            self.domain, JIRA_API_BASE_PATH, end_point
        );
        let jql_info = query
            .and_then(|q| q.iter().find(|(k, _)| *k == "jql"))
            .map(|(_, v)| format!(" | JQL: {}", v))
            .unwrap_or_default();
        println!("API call to Jira: {}{}", &url, jql_info);

        let mut request = self
            .http
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json");

        if let Some(q) = query {
            request = request.query(q);
        }

        let response = request
            .send()
            .await
            .map_err(|e| WorkOsError::Jira(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(WorkOsError::Jira(format!(
                "Request failed: {} - {}",
                status, body
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| WorkOsError::Jira(format!("JSON parse error: {}", e)))
    }

    async fn get_user_info(&mut self, account_id: &str) -> Result<JiraUser> {
        if let Some(user) = self.user_cache.get(account_id) {
            return Ok(user.clone());
        }

        let endpoint = format!("user?accountId={}", account_id);
        let user: JiraUser = match self.get(&endpoint, None).await {
            Ok(u) => u,
            Err(_) => JiraUser::unknown(account_id),
        };

        self.user_cache.insert(account_id.to_string(), user.clone());
        Ok(user)
    }

    async fn replace_user_mentions(&mut self, text: &str) -> Result<String> {
        let re = Regex::new(r"\[~accountid:([^\]]+)\]").unwrap();
        let mut result = text.to_string();
        let matches: Vec<(String, String)> = re
            .captures_iter(text)
            .map(|cap| {
                let full_match = cap.get(0).unwrap().as_str().to_string();
                let account_id = cap[1].to_string();
                (full_match, account_id)
            })
            .collect();

        for (full_match, account_id) in matches {
            let user = self.get_user_info(&account_id).await?;
            let handle = format!("@{}", user.display_name);
            result = result.replace(&full_match, &handle);
        }

        Ok(result)
    }
}

// ============================
// Helpers
// ============================

fn issue_to_message(issue: JiraIssue, domain: &str, default_priority: Priority) -> Message {
    let url = issue.url(domain);
    let description = build_description(&issue);

    let message_type = match issue.fields.issue_type.name.to_lowercase().as_str() {
        "bug" => MessageType::Issue,
        _ => MessageType::Ticket,
    };

    let priority = issue
        .fields
        .priority
        .as_ref()
        .map(|p| JiraPriority::map_priority(Some(&p.name)))
        .unwrap_or(default_priority);

    let status =
        JiraStatusCategory::map_status_category(issue.fields.status.status_category.as_ref());

    let created_at = parse_datetime(&issue.fields.created);
    let updated_at = parse_datetime(&issue.fields.updated);

    let title = format!("[{}] {}", issue.key, issue.fields.summary);

    let mut message = Message::new("jira", message_type, &issue.key, title, url)
        .with_description(description)
        .with_priority(priority)
        .with_status(status)
        .with_date(created_at, updated_at);

    if let Some(assignee) = issue.fields.assignee {
        message = message.with_person(Person {
            name: assignee.display_name,
            username: assignee.account_id,
            role: PersonRole::Assignee,
        });
    }

    message
}

fn build_description(issue: &JiraIssue) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "Project: {} | Type: {}",
        issue.fields.project.key, issue.fields.issue_type.name
    ));
    lines.push(format!("Status: {}", issue.fields.status.name));

    let created_ts = parse_jira_datetime(&issue.fields.created)
        .map(|dt| dt.with_timezone(&Local).format("%b %d, %l:%M %p").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let updated_ts = parse_jira_datetime(&issue.fields.updated)
        .map(|dt| dt.with_timezone(&Local).format("%b %d, %l:%M %p").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    lines.push(format!("Created: {} | Updated: {}", created_ts, updated_ts));

    let assignee = issue
        .fields
        .assignee
        .as_ref()
        .map(|a| a.display_name.as_str())
        .unwrap_or("Unassigned");
    let reporter = issue
        .fields
        .reporter
        .as_ref()
        .map(|r| r.display_name.as_str())
        .unwrap_or("Unknown");
    lines.push(format!("Assigned: {} | Reporter: {}", assignee, reporter));

    // sprint info
    if let Some(sprint) = issue.fields.extract_sprint() {
        lines.push(format!("Sprint: {}", sprint.display()));
    }

    // parent message
    if let Some((parent_task_key, parent_task_name)) = issue.fields.extract_parent_task() {
        lines.push(format!(
            "Parent message: {} - {}",
            parent_task_key, parent_task_name
        ));
    }

    // epic info
    if let Some((epic_key, epic_name)) = issue.fields.extract_epic() {
        if epic_name.is_empty() {
            lines.push(format!("Epic: {}", epic_key));
        } else {
            lines.push(format!("Epic: {} - {}", epic_key, epic_name));
        }
    }

    if let Some(ref due) = issue.fields.due_date {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(due, "%Y-%m-%d") {
            lines.push(format!("Due: {}", date.format("%b %d, %Y")));
        }
    }

    // activity log
    if let Some(changelog) = issue.changelog.as_ref() {
        let recent: Vec<String> = changelog
            .histories
            .iter()
            .take(6)
            .map(|e| {
                let summary = e.summary();
                if summary.chars().count() > 50 {
                    return None;
                }

                let ts = parse_jira_datetime(&e.created)
                    .map(|dt| dt.with_timezone(&Local).format("%b %d, %l:%M %p").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                Some(format!("→ {}: {} ({})", ts, summary, e.author.display_name))
            })
            .flatten()
            .collect();

        if !recent.is_empty() {
            lines.push(String::new());
            lines.push("Recent Activity:".to_string());
            lines.extend(recent);
        }
    }

    // comments
    if let Some(ref comment) = issue.fields.comment {
        let recent: Vec<String> = comment
            .comments
            .iter()
            .take(6)
            .map(|c| {
                let ts = parse_jira_datetime(&c.created)
                    .map(|dt| dt.with_timezone(&Local).format("%b %d, %l:%M %p").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let body = c.body_text();
                let body_short = if body.len() > 100 {
                    format!("{}...", &body[..100])
                } else {
                    body
                };
                format!("[{}] {}: {}", ts, c.author.display_name, body_short)
            })
            .collect();

        if !recent.is_empty() {
            lines.push(String::new());
            lines.push("Comments:".to_string());
            lines.extend(recent);
        }
    }

    lines.join("\n")
}

// ============================
// Utils
// ============================

fn parse_datetime(s: &str) -> DateTime<Utc> {
    parse_jira_datetime(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now())
}

// fixed using AI this one .....!!!!! Need better solution
fn parse_jira_datetime(s: &str) -> Option<DateTime<FixedOffset>> {
    // Try RFC 3339 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt);
    }

    // Jira uses format like "2026-02-04T06:50:42.455+0000" (no colon in timezone)
    // Try to normalize by inserting colon: +0000 -> +00:00
    let normalized = if s.len() >= 5 {
        let (prefix, tz) = s.split_at(s.len() - 5);
        if (tz.starts_with('+') || tz.starts_with('-')) && !tz.contains(':') {
            format!("{}{}:{}", prefix, &tz[..3], &tz[3..])
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    };

    DateTime::parse_from_rfc3339(&normalized).ok()
}

fn deduplicate_messages(messages: Vec<Message>) -> Vec<Message> {
    let mut seen = HashSet::new();
    messages
        .into_iter()
        .filter(|t| seen.insert(t.id.clone()))
        .collect()
}
