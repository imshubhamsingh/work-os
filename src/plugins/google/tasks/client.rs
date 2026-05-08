use chrono::{DateTime, Utc};
use reqwest::Client;

use crate::core::message::{Message, MessageStatus, MessageType, Priority};
use crate::error::{Result, WorkOsError};
use crate::plugins::google::auth::{refresh_access_token, GoogleOAuthConfig};
use crate::plugins::google::tasks::model::*;

const TASKS_API_BASE: &str = "https://tasks.googleapis.com/tasks/v1";

pub struct GoogleTasksClient {
    http: Client,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
}

impl GoogleTasksClient {
    pub fn new(config: &GoogleOAuthConfig) -> Self {
        Self {
            http: Client::new(),
            access_token: config.access_token.clone(),
            refresh_token: config.refresh_token.clone(),
            expires_at: config.expires_at,
        }
    }

    async fn fresh_token(&self) -> crate::error::Result<String> {
        let needs_refresh = self
            .expires_at
            .map(|exp| exp < chrono::Utc::now().timestamp() + 300)
            .unwrap_or(false);

        if needs_refresh {
            if let Some(ref rt) = self.refresh_token {
                return refresh_access_token(rt).await;
            }
        }

        Ok(self.access_token.clone())
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/users/@me/lists", TASKS_API_BASE);
        println!("API call to Google Tasks: {}", &url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| WorkOsError::Google(format!("Connection test failed: {}", e)))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            if let Some(ref rt) = self.refresh_token {
                let new_token = refresh_access_token(rt).await?;
                let resp = self
                    .http
                    .get(&url)
                    .bearer_auth(&new_token)
                    .send()
                    .await
                    .map_err(|e| WorkOsError::Google(format!("Connection test failed after refresh: {}", e)))?;
                return Ok(resp.status().is_success());
            }
        }

        Ok(resp.status().is_success())
    }

    // ============================
    // Messages
    // ============================

    pub async fn get_all_messages(&self) -> Result<Vec<Message>> {
        let token = self.fresh_token().await?;
        let url = format!("{}/users/@me/lists", TASKS_API_BASE);
        println!("API call to Google Tasks: {}", &url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| WorkOsError::Google(format!("Failed to fetch task lists: {}", e)))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WorkOsError::Google(format!("Tasks API error: {}", body)));
        }

        let lists: TaskListsResponse = resp
            .json()
            .await
            .map_err(|e| WorkOsError::Google(format!("Failed to parse task lists: {}", e)))?;

        let mut messages = Vec::new();

        for list in lists.items.unwrap_or_default() {
            let tasks = self.fetch_tasks_for_list(&list.id, &token).await?;
            for task in tasks {
                if let Some(msg) = build_message(task, &list.title) {
                    messages.push(msg);
                }
            }
        }

        Ok(messages)
    }

    // ============================
    // Google Tasks API
    // ============================

    async fn fetch_tasks_for_list(&self, list_id: &str, token: &str) -> Result<Vec<TaskItem>> {
        let url = format!("{}/lists/{}/tasks", TASKS_API_BASE, list_id);
        println!("API call to Google Tasks: {}", &url);
        let resp = self
            .http
            .get(&url)
            .query(&[
                ("showCompleted", "false"),
                ("showDeleted", "false"),
                ("showHidden", "false"),
            ])
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| WorkOsError::Google(format!("Failed to fetch tasks: {}", e)))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WorkOsError::Google(format!("Tasks API error: {}", body)));
        }

        let data: TasksResponse = resp
            .json()
            .await
            .map_err(|e| WorkOsError::Google(format!("Failed to parse tasks: {}", e)))?;

        Ok(data.items.unwrap_or_default())
    }
}

// ============================
// Helpers
// ============================

fn build_message(task: TaskItem, list_name: &str) -> Option<Message> {
    let title = task.title?.trim().to_string();
    if title.is_empty() {
        return None;
    }

    let status = MessageStatus::Open;

    let priority = match task.due.as_deref() {
        Some(due_str) => match due_str.parse::<DateTime<Utc>>() {
            Ok(due_dt) => {
                let days_until = (due_dt.timestamp() - Utc::now().timestamp()) / 86400;
                if days_until < 0 {
                    Priority::High
                } else if days_until == 0 {
                    Priority::High
                } else if days_until <= 3 {
                    Priority::Medium
                } else {
                    Priority::Low
                }
            }
            Err(_) => Priority::Low,
        },
        None => Priority::Low,
    };

    let mut desc_lines: Vec<String> = Vec::new();
    desc_lines.push(format!("List: {}", list_name));

    if task.parent.is_some() {
        desc_lines.push("Subtask".to_string());
    }
    if let Some(ref notes) = task.notes {
        let notes = notes.trim();
        if !notes.is_empty() {
            desc_lines.push(format!("Notes: {}", notes));
        }
    }
    if let Some(ref due_str) = task.due {
        if let Ok(due_dt) = due_str.parse::<DateTime<Utc>>() {
            let local = due_dt.with_timezone(&chrono::Local);
            desc_lines.push(format!("Due: {}", local.format("%b %d, %Y")));
        }
    }

    let updated_at = task
        .updated
        .as_deref()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .unwrap_or_else(Utc::now);

    let url = task.self_link.unwrap_or_default();

    Some(
        Message::new(
            "google_tasks",
            MessageType::GoogleTask,
            &format!("gtask:{}", task.id),
            title,
            url,
        )
        .with_description(desc_lines.join("\n"))
        .with_priority(priority)
        .with_status(status)
        .with_date(updated_at, updated_at),
    )
}
