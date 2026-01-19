use regex::Regex;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
// use regex::Regex;
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use std::fmt::Write;

use crate::core::task::{
    Person, PersonRole, Priority, SlackMetadata, Task, TaskMetadata, TaskType,
};
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

        let dms = self.get_all_dms().await?;
        all_tasks.extend(dms);

        let group_dms = self.get_all_group_dms().await?;
        all_tasks.extend(group_dms);

        let channel_messages = self.get_all_channel_messages().await?;
        all_tasks.extend(channel_messages);

        let mentions_messages = self.get_all_mentions().await?;
        all_tasks.extend(mentions_messages);

        all_tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(all_tasks)
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
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        println!("{}", after_date);
        let search_message_url =
            format!("search.messages?query=<@{}> after:{}", user_id, after_date);
        println!("URL {}", search_message_url);
        let mentioned_messages: SlackResponse<SlackSearch> = self.get(&search_message_url).await?;

        if !mentioned_messages.ok {
            return Ok(Vec::new());
        }

        let matches = mentioned_messages
            .data
            .map(|d| d.messages.matches)
            .unwrap_or_default();
        println!("Len {}", matches.len());
        if matches.is_empty() {
            return Ok(Vec::new());
        }

        for result in matches.iter() {
            println!("{}", result.kind);
            let updated_at: DateTime<Utc> = DateTime::parse_from_str(&result.ts, "%s.%f")
                .unwrap()
                .into();

            let author_name = match self.get_user_info(&result.user).await? {
                Some(user) => user.name,
                _ => continue,
            };

            let formatted_text = self.replace_user_id_with_handle(&result.text).await?;
            let mut description = format!("{}: {}", author_name, formatted_text);

            if let Some(parent_thread_ts) = extract_parent_ts(&result.permalink) {
                if (parent_thread_ts != result.ts) {
                    let thread_message_url = format!(
                        "conversations.replies?channel={}&ts={}&limit=1000",
                        result.channel.id, parent_thread_ts,
                    );
                    println!("{}", thread_message_url);
                    let thread_messages: SlackResponse<SlackThread> =
                        self.get(&thread_message_url).await?;
                    match serde_json::to_string_pretty(&thread_messages) {
                        Ok(json) => println!("{}", json),
                        _ => println!("nothing"),
                    }

                    let mut thread = thread_messages.data.map(|d| d.messages).unwrap_or_default();

                    println!("thread length ----- {}", thread.len());
                    let _ = writeln!(
                        description,
                        "\nThread messages (first and last 6 messages if present): ┐",
                    );
                    if thread.len() > 6 {
                        let mut trimmed = Vec::with_capacity(6);

                        // first 3
                        trimmed.extend(thread.drain(..3));

                        // last 3
                        trimmed.extend(thread.drain(thread.len() - 3..));

                        thread = trimmed;
                    }

                    for t in thread.iter() {
                        if let Some(author) = self.get_user_info(&t.user).await? {
                            let t_formate_message =
                                self.replace_user_id_with_handle(&t.text).await?;
                            let _ = writeln!(description, "{}:{}", author.name, &t_formate_message);
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

            let task = Task::new(
                "slack",
                TaskType::Message,
                &result.channel.id,
                format!("Mention in {}", channel_name),
                result.permalink.clone(),
            )
            .with_description(description)
            .with_date(updated_at, updated_at);

            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_all_channel_messages(&mut self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        let provied_channels = self.channels.clone();

        for channel_id in provied_channels {
            let messages = self.get_channel_messages(&channel_id).await?;

            if messages.is_empty() {
                continue;
            }

            let mut description = String::new();

            for message in messages.iter().rev() {
                let author_id = match message.user.as_deref() {
                    Some(id) => id,
                    _ => continue,
                };

                if let Some(author) = self.get_user_info(author_id).await? {
                    use std::fmt::Write;
                    let formated_message = self.replace_user_id_with_handle(&message.text).await?;
                    let _ = writeln!(description, "{}:{}", author.name, &formated_message);
                }
            }

            let update_at: DateTime<Utc> =
                DateTime::parse_from_str(&messages[messages.len() - 1].ts, "%s.%f")
                    .unwrap()
                    .into();

            let task = Task::new(
                "slack",
                TaskType::Message,
                &channel_id,
                "testing_channel".to_string(),
                format!("https://slack.com/archives/{}", &channel_id),
            )
            .with_date(update_at, update_at)
            .with_description(description.trim_end().to_string());

            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_relevant_channels(&self, types: &[&str]) -> Result<Vec<Channel>> {
        let url = format!("conversations.list?types={}&limit=1000", types.join("&"));
        let response: SlackResponse<ConversationsListData> = self.get(&url).await?;

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

    async fn get_all_group_dms(&mut self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let channels = self.get_relevant_channels(&["mpim"]).await?;

        for channel in channels {
            let messages = self.get_channel_messages(&channel.id).await?;

            if messages.is_empty() {
                continue;
            }

            let mut description = String::new();

            for message in messages.iter().rev() {
                let author_id = match message.user.as_deref() {
                    Some(id) => id,
                    _ => continue,
                };
                if let Some(author) = self.get_user_info(author_id).await? {
                    use std::fmt::Write;
                    let _ = writeln!(description, "{}:{}", author.name, message.text);
                }
            }

            println!("Description: {}", description);

            let update_at: DateTime<Utc> =
                DateTime::parse_from_str(&messages[messages.len() - 1].ts, "%s.%f")
                    .unwrap()
                    .into();

            let task = Task::new(
                "slack",
                TaskType::Message,
                &channel.id,
                channel
                    .purpose
                    .as_ref()
                    .map(|p| p.value.clone())
                    .unwrap_or("Unknow Group DM".to_string()),
                format!("https://slack.com/archives/{}", channel.id),
            )
            .with_date(update_at, update_at)
            .with_description(description.trim_end().to_string());

            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_all_dms(&mut self) -> Result<Vec<Task>> {
        let channels = self.get_relevant_channels(&["im"]).await?;

        let mut tasks = Vec::new();

        for channel in channels {
            let messages = self.get_channel_messages(&channel.id).await?;
            if messages.is_empty() {
                continue;
            }

            let user_id = match channel.user.as_deref() {
                Some(id) => id,
                None => continue,
            };

            let user = match self.get_user_info(user_id).await? {
                Some(u) if !u.deleted && !u.is_bot => u,
                _ => continue,
            };

            let real_name = user.real_name.clone().unwrap_or_else(|| user.name.clone());

            let mut description = String::new();

            for msg in messages.iter().rev() {
                let author_id = match msg.user.as_deref() {
                    Some(id) => id,
                    None => continue,
                };

                if let Some(author) = self.get_user_info(author_id).await? {
                    use std::fmt::Write;
                    let formate_message = self.replace_user_id_with_handle(&msg.text).await?;
                    println!("{}", &formate_message);
                    let _ = writeln!(description, "{}: {}", author.name, &formate_message);
                }
            }

            if description.is_empty() {
                continue;
            }

            let updated_at: DateTime<Utc> =
                DateTime::parse_from_str(&messages[messages.len() - 1].ts, "%s.%f")
                    .unwrap()
                    .into();

            let task = Task::new(
                "slack",
                TaskType::Message,
                &channel.id,
                format!("DM between you and {}", real_name),
                format!("https://slack.com/archives/{}", channel.id),
            )
            .with_date(updated_at, updated_at)
            .with_description(description.trim_end().to_string());

            tasks.push(task);
        }

        Ok(tasks)
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

    async fn replace_user_id_with_handle(&mut self, description: &str) -> Result<String> {
        let reg = Regex::new(r"<@([A-Z0-9]+)(?:\|[^>]+)?>").unwrap();
        let mut result = description.to_string();

        for cap in reg.captures_iter(description) {
            let user_id = &cap[1];
            let mention = format!("<@{}>", user_id);
            let full_match = cap.get(0).unwrap().as_str();
            println!("mention {}", &mention);
            if let Some(user) = self.get_user_info(user_id).await? {
                let handle = format!("@{}", user.name);
                result = result.replace(full_match, &handle)
            }
        }

        Ok(result)
    }

    async fn get_user_info(&mut self, user_id: &str) -> Result<Option<SlackUser>> {
        if let Some(cached) = self.user_cache.get(user_id) {
            return Ok(cached.clone());
        }

        let url = format!("users.info?user={}", user_id);
        let response: SlackResponse<UsersInfoData> = match self.get(&url).await {
            Ok(resp) => resp,
            Err(err) => {
                println!("Slack request failed for {}: {}", user_id, err);
                self.user_cache.insert(user_id.to_string(), None);
                return Ok(None);
            }
        };

        if !response.ok {
            let slack_error = response
                .error
                .unwrap_or_else(|| "unknown_slack_error".into());

            println!("Slack returned error for {}: {}", user_id, slack_error);
            self.user_cache.insert(user_id.to_string(), None);
            return Ok(None);
        }

        let Some(user) = response.data.map(|d| d.user) else {
            println!("Slack returned ok but no user payload for {}", user_id);
            self.user_cache.insert(user_id.to_string(), None);
            return Ok(None);
        };

        self.user_cache
            .insert(user_id.to_string(), Some(user.clone()));

        Ok(Some(user))
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
