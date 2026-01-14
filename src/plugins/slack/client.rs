use std::collections::HashMap;

use chrono::Utc;
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::core::task::{Person, PersonRole, Priority, SlackMetadata, Task, TaskMetadata, TaskType};
use crate::error::{Result, WorkOsError};
use crate::models::config::SlackConfig;
use crate::plugins::slack::model::*;

const SLACK_API_BASE_URL: &str = "https://slack.com/api";
pub struct SlackClient {
    http: Client,
    token: String,
    keywords: Vec<String>,
    channels: Vec<String>,
    max_messages_per_channel: usize,
    user_cache: HashMap<String, SlackUser>,
}

impl SlackClient {
    pub fn new(config: &SlackConfig) -> Result<Self> {
        let http = Client::new();
        Ok(Self {
            http,
            token: config.token.clone(),
            keywords: config.keywords.clone(),
            channels: config.channels.clone(),
            max_messages_per_channel: config.max_messages_per_channel,
            user_cache: HashMap::new(),
        })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let response: SlackResponse<AuthTestData> = self.get("auth.test").await?;
        Ok(response.ok)
    }

    pub async fn get_all_tasks(&mut self) -> Result<Vec<Task>> {
        let mut all_tasks = Vec::new();

        let channels = self.get_relevant_channels().await?;
        for channel in channels {
            let messages = self.get_channel_messages(&channel.id).await?;
            for message in messages {
                if let Some(task) = self.message_to_task(&channel, &message).await {
                    all_tasks.push(task);
                }
            }
        }

        all_tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(all_tasks)
    }

    async fn get_relevant_channels(&self) -> Result<Vec<Channel>> {
        let response: SlackResponse<ConversationsListData> = self
            .get("conversations.list?types=im&limit=1000")
            .await?;

        if !response.ok {
            return Err(WorkOsError::Slack(
                response.error.unwrap_or("Unknown error".to_string()),
            ));
        }


        let channels = response.data.map(|d| d.channels).unwrap_or_default();

        let filtered_channels = channels
            .into_iter()
            .filter(|c| c.is_member || c.is_im)
            .filter(|c| self.channels.is_empty() || self.channels.contains(&c.id))
            .collect();

        Ok(filtered_channels)
    }

    async fn get_channel_messages(&self, channel_id: &str) -> Result<Vec<SlackMessage>> {
        
        let (oldest_timestamp, newest_timestamp) = last_24_hr_range();

        let url = format!(
            "conversations.history?channel={}&limit={}&oldest={}&newest={}",
            channel_id, self.max_messages_per_channel, oldest_timestamp, newest_timestamp
        );

        let response: SlackResponse<ConversationsHistoryData> = self.get(&url).await?;

        if !response.ok {
            return Ok(Vec::new());
        }

        Ok(response.data.map(|d| d.messages).unwrap_or_default())
    }

    async fn message_to_task(&mut self, channel: &Channel, message: &SlackMessage) -> Option<Task> {
        let author = if let Some(ref user_id) = message.user {
            self.get_user_info(&user_id).await.ok()
        } else {
            None
        };

        println!("Author: {:?}", author);

        let title = if message.text.len() > 80 {
            format!("{}...", &message.text.chars().take(80).collect::<String>())
        } else {
            message.text.clone()
        };

        let url = format!(
            "https://slack.com/archives/{}/p{}",
            channel.id, message.ts
        );

        let mut task = Task::new("slack", TaskType::Message, &message.ts, title, url)
            .with_priority(Priority::Unknown)
            .with_description(message.text.clone())
            .with_metadata(TaskMetadata::Slack(SlackMetadata {
                channel: channel.id.clone(),
                thread_ts: message.ts.clone(),
            }));

        if let Some(author) = author {
            let real_name = author.real_name.unwrap_or_else(|| author.name.clone());
            task = task.with_person(Person {
                name: real_name.clone(),
                username: real_name.clone(),
                role: PersonRole::Author,
            });
        }

        Some(task)
    }

    async fn get_user_info(&mut self, user_id: &str) -> Result<SlackUser> {
        if let Some(user) = self.user_cache.get(user_id) {
            return Ok(user.clone());
        }

        let url = format!("users.info?user={}", user_id);
        let response: SlackResponse<UsersInfoData> = self.get(&url).await?;

        if !response.ok {
            return Err(WorkOsError::Slack(
                response
                    .error
                    .unwrap_or_else(|| "User not found".to_string()),
            ));
        }
        let user = response
            .data
            .map(|d| d.user)
            .ok_or_else(|| WorkOsError::Slack("No user data found".to_string()))?;
        self.user_cache.insert(user_id.to_string(), user.clone());
        Ok(user)
    }

    async fn get<T: DeserializeOwned>(&self, end_point: &str) -> Result<T> {
        let url = format!("{}/{}", SLACK_API_BASE_URL, end_point);
        let response = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| WorkOsError::Slack(e.to_string()))?;

        let data = response
            .json::<T>()
            .await
            .map_err(|e| WorkOsError::Slack(format!("JSON parse error: {}", e)))?;

        Ok(data)
    }
}

fn last_24_hr_range() -> (i64, i64) {
    let now = Utc::now().timestamp();
    let day_ago  = now - 86400;
    (day_ago, now)
}