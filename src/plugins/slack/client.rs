use regex::Regex;
use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use std::fmt::Write;

use crate::core::task::{Task, TaskType};
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
    user_cache: HashMap<String, Option<SlackUser>>,
    channel_cache: HashMap<String, Option<SlackChannel>>,
}

impl SlackClient {
    pub fn new(config: &SlackConfig) -> Result<Self> {
        Ok(Self {
            http: Client::new(),
            token: config.token.clone(),
            keywords: config.keywords.clone(),
            channels: config.channels.clone(),
            max_messages_per_channel: config.max_messages_per_channel,
            user_cache: HashMap::new(),
            channel_cache: HashMap::new(),
        })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let response: SlackResponse<AuthTestData> = self.get("auth.test").await?;
        Ok(response.ok)
    }

    pub async fn get_all_tasks(&mut self) -> Result<Vec<Task>> {
        let mut all_tasks = Vec::new();

        all_tasks.extend(self.get_all_dms().await?);
        all_tasks.extend(self.get_all_group_dms().await?);
        all_tasks.extend(self.get_all_channel_messages().await?);
        all_tasks.extend(self.get_all_mentions().await?);

        all_tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(all_tasks)
    }

    // ============================
    // Tasks
    // ============================

    async fn get_all_dms(&mut self) -> Result<Vec<Task>> {
        let channels = self.get_relevant_channels(&["im"]).await?;
        let mut tasks = Vec::new();

        for channel in channels {
            let messages = self.get_channel_messages(&channel.id).await?;
            if messages.is_empty() {
                continue;
            }

            let Some(user_id) = channel.user.as_deref() else {
                continue;
            };
            let Some(user) = self.get_valid_user(user_id).await? else {
                continue;
            };

            let real_name = user.real_name.clone().unwrap_or_else(|| user.name.clone());

            let description = self.build_description_from_messages(&messages).await?;

            if description.is_empty() {
                continue;
            }

            let updated_at = latest_message_ts(&messages);

            let task = Self::build_task(
                &channel.id,
                format!("DM between you and {}", real_name),
                format!("https://slack.com/archives/{}", channel.id),
                description,
                updated_at,
            );

            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_all_group_dms(&mut self) -> Result<Vec<Task>> {
        let channels = self.get_relevant_channels(&["mpim"]).await?;
        let mut tasks = Vec::new();

        for channel in channels {
            let messages = self.get_channel_messages(&channel.id).await?;
            if messages.is_empty() {
                continue;
            }

            let description = self.build_description_from_messages(&messages).await?;

            let updated_at = latest_message_ts(&messages);

            let title = channel
                .purpose
                .as_ref()
                .map(|p| p.value.clone())
                .unwrap_or_else(|| "Unknow Group DM".to_string());

            let task = Self::build_task(
                &channel.id,
                title,
                format!("https://slack.com/archives/{}", channel.id),
                description,
                updated_at,
            );

            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_all_channel_messages(&mut self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        for channel_id in self.channels.clone() {
            let messages = self.get_channel_messages(&channel_id).await?;
            if messages.is_empty() {
                continue;
            }

            let description = self.build_description_from_messages(&messages).await?;

            let updated_at = latest_message_ts(&messages);

            let channel_name: String = self
                .get_channel_info(&channel_id)
                .await?
                .and_then(|c| Some(c.name))
                .unwrap_or_else(|| channel_id.clone());

            let task = Self::build_task(
                &channel_id,
                channel_name,
                format!("https://slack.com/archives/{}", channel_id),
                description,
                updated_at,
            );

            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_all_mentions(&mut self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let current_user: SlackResponse<AuthTestData> = self.get("auth.test").await?;

        let user_id = current_user
            .data
            .expect("auth.test must return data")
            .user_id;

        let (oldest_timestamp, _) = last_24_hr_range();
        let after_date = DateTime::from_timestamp(oldest_timestamp, 0)
            // @todo fix this, some time zone issue.
            .map(|dt| (dt - Duration::hours(24)).format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        let before_date = DateTime::from_timestamp(oldest_timestamp, 0)
            .map(|dt| (dt + Duration::hours(24)).format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        let search_message_url =
            format!("search.messages?query=<@{}> after:{} before:{}", user_id, after_date, before_date);
        let mentioned_messages: SlackResponse<SlackSearch> = self.get(&search_message_url).await?;

        if !mentioned_messages.ok {
            return Ok(Vec::new());
        }

        let matches = mentioned_messages
            .data
            .map(|d| d.messages.matches)
            .unwrap_or_default();

        if matches.is_empty() {
            return Ok(Vec::new());
        }

        for result in matches.iter() {
            let updated_at = parse_ts(&result.ts);

            let Some(author) = self.get_user_info(&result.user).await? else {
                continue;
            };

            let formatted_text = self.replace_user_id_with_handle(&result.text).await?;
            let mut description = format!("{}: {}", author.name, formatted_text);

            if let Some(parent_thread_ts) = extract_parent_ts(&result.permalink) {
                if parent_thread_ts != result.ts {
                    let thread_message_url = format!(
                        "conversations.replies?channel={}&ts={}&limit=1000",
                        result.channel.id, parent_thread_ts,
                    );

                    let thread_messages: SlackResponse<SlackThread> =
                        self.get(&thread_message_url).await?;

                    let mut thread = thread_messages.data.map(|d| d.messages).unwrap_or_default();

                    let _ = writeln!(
                        description,
                        "\nThread messages (first and last 6 messages if present): ┐",
                    );

                    if thread.len() > 6 {
                        let mut trimmed = Vec::with_capacity(6);
                        trimmed.extend(thread.drain(..3));
                        trimmed.extend(thread.drain(thread.len() - 3..));
                        thread = trimmed;
                    }

                    for t in &thread {
                        if let Some(author) = self.get_user_info(&t.user).await? {
                            let msg = self.replace_user_id_with_handle(&t.text).await?;
                            let _ = writeln!(description, "{}: {}", author.name, msg);
                        }
                    }

                    if let Some(first) = thread.first() {
                        let _ = writeln!(
                            description,
                            "Total messages in thread: {}",
                            first.reply_count.unwrap_or(0)
                        );
                    }
                }
            }

            let channel_name = result
                .channel
                .name
                .starts_with("U")
                .then(|| format!("DM {}", result.username.as_deref().unwrap_or("unknown")))
                .unwrap_or_else(|| result.channel.name.clone());

            let task = Self::build_task(
                &result.channel.id,
                format!("Mention in {}", channel_name),
                result.permalink.clone(),
                description,
                updated_at,
            );

            tasks.push(task);
        }

        Ok(tasks)
    }

    // ============================
    // Slack API
    // ============================

    async fn get_relevant_channels(&self, types: &[&str]) -> Result<Vec<SlackChannel>> {
        let url = format!("conversations.list?types={}&limit=1000", types.join("&"));
        let response: SlackResponse<ConversationsListData> = self.get(&url).await?;

        if !response.ok {
            return Err(WorkOsError::Slack(
                response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let channels = response.data.map(|d| d.channels).unwrap_or_default();

        Ok(channels
            .into_iter()
            .filter(|c| c.is_member || c.is_im)
            .filter(|c| self.channels.is_empty() || self.channels.contains(&c.id))
            .collect())
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

    async fn get_channel_info(&mut self, channel_id: &str) -> Result<Option<SlackChannel>> {
        if let Some(cached) = self.channel_cache.get(channel_id) {
            return Ok(cached.clone());
        }

        let url = format!("conversations.info?channel={}", channel_id);
        let response: SlackResponse<ConversationsInfoData> = match self.get(&url).await {
            Ok(resp) => resp,
            Err(_) => {
                self.channel_cache.insert(channel_id.to_string(), None);
                return Ok(None);
            }
        };

        if !response.ok {
            self.channel_cache.insert(channel_id.to_string(), None);
            return Ok(None);
        }

        let channel = response.data.map(|d| d.channel);

        self.channel_cache
            .insert(channel_id.to_string(), channel.clone());

        Ok(channel)
    }

    async fn get_user_info(&mut self, user_id: &str) -> Result<Option<SlackUser>> {
        if let Some(cached) = self.user_cache.get(user_id) {
            return Ok(cached.clone());
        }

        let url = format!("users.info?user={}", user_id);
        let response: SlackResponse<UsersInfoData> = match self.get(&url).await {
            Ok(resp) => resp,
            Err(_) => {
                self.user_cache.insert(user_id.to_string(), None);
                return Ok(None);
            }
        };

        if !response.ok {
            self.user_cache.insert(user_id.to_string(), None);
            return Ok(None);
        }

        let Some(user) = response.data.map(|d| d.user) else {
            self.user_cache.insert(user_id.to_string(), None);
            return Ok(None);
        };

        self.user_cache
            .insert(user_id.to_string(), Some(user.clone()));

        Ok(Some(user))
    }

    async fn get<T: DeserializeOwned>(&self, end_point: &str) -> Result<T> {
        println!("API call to Slack: {}", &end_point);
        let url = format!("{}/{}", SLACK_API_BASE_URL, end_point);
        let response = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| WorkOsError::Slack(e.to_string()))?;

        response
            .json::<T>()
            .await
            .map_err(|e| WorkOsError::Slack(format!("JSON parse error: {}", e)))
    }

    // ============================
    // Helpers
    // ============================

    async fn get_valid_user(&mut self, user_id: &str) -> Result<Option<SlackUser>> {
        match self.get_user_info(user_id).await? {
            Some(u) if !u.deleted && !u.is_bot => Ok(Some(u)),
            _ => Ok(None),
        }
    }

    async fn replace_user_id_with_handle(&mut self, description: &str) -> Result<String> {
        let reg = Regex::new(r"<@([A-Z0-9]+)(?:\|[^>]+)?>").unwrap();
        let mut result = description.to_string();

        for cap in reg.captures_iter(description) {
            let user_id = &cap[1];
            let full_match = cap.get(0).unwrap().as_str();

            if let Some(user) = self.get_user_info(user_id).await? {
                let handle = format!("@{}", user.name);
                result = result.replace(full_match, &handle)
            }
        }

        Ok(result)
    }

    async fn build_description_from_messages(
        &mut self,
        messages: &[SlackMessage],
    ) -> Result<String> {
        let mut description = String::new();

        for msg in messages.iter().rev() {
            let Some(author_id) = msg.user.as_deref() else {
                continue;
            };
            let author = match self.get_user_info(author_id).await? {
                Some(a) => a,
                _ => SlackUser {
                    id: "-1".to_string(),
                    name: format!("Unknown user {}", author_id),
                    real_name: None,
                    deleted: false,
                    is_bot: false,
                },
            };

            let text = self.replace_user_id_with_handle(&msg.text).await?;

            let _ = writeln!(description, "{}: {}", author.name, text);
        }

        Ok(description.trim_end().to_string())
    }

    fn build_task(
        channel_id: &str,
        title: String,
        url: String,
        description: String,
        updated_at: DateTime<Utc>,
    ) -> Task {
        Task::new("slack", TaskType::Message, channel_id, title, url)
            .with_date(updated_at, updated_at)
            .with_description(description)
    }
}

// ============================
// Utils
// ============================

fn latest_message_ts(messages: &[SlackMessage]) -> DateTime<Utc> {
    parse_ts(&messages[messages.len() - 1].ts)
}

fn parse_ts(ts: &str) -> DateTime<Utc> {
    DateTime::parse_from_str(ts, "%s.%f").unwrap().into()
}

fn last_24_hr_range() -> (i64, i64) {
    let now = Utc::now().timestamp();
    let day_ago = now - 86400;
    let go_back_by_day: i64 = 0 * 86400;
    (day_ago - go_back_by_day, now)
}

fn extract_parent_ts(permalink: &str) -> Option<String> {
    let url = Url::parse(permalink).ok()?;
    url.query_pairs()
        .find(|(key, _)| key == "thread_ts")
        .map(|(_, value)| value.to_string())
}
