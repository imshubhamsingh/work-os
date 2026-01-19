use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackResponse<T> {
    pub ok: bool,

    #[serde(default)]
    pub error: Option<String>,

    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_member: bool,
    #[serde(default)]
    pub is_group: bool,
    #[serde(default)]
    pub is_im: bool,
    #[serde(default)]
    pub is_mpim: bool,
    #[serde(default)]
    pub user: Option<String>,
    pub purpose: Option<Purpose>,
}
#[derive(Debug, Deserialize)]
pub struct Purpose {
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub last_set: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackReaction {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub text: String,
    #[serde(default)]
    pub user: Option<String>,
    pub ts: String,
    #[serde(default)]
    pub thread_ts: Option<String>,
    #[serde(default)]
    pub reply_count: u32,
    #[serde(default)]
    pub reactions: Option<Vec<SlackReaction>>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationsListData {
    pub channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationsHistoryData {
    pub messages: Vec<SlackMessage>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct UsersInfoData {
    pub user: SlackUser,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackUser {
    pub id: String,
    pub name: String,
    pub real_name: Option<String>,
    pub deleted: bool,
    pub is_bot: bool,
}

#[derive(Debug, Deserialize)]
pub struct AuthTestData {
    pub user_id: String,
    pub user: String,
    pub team: String,
}

#[derive(Debug, Deserialize)]
pub struct SlackSearch {
    pub messages: SlackSearchMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackSearchMessage {
    pub matches: Vec<SlackSearchMessageMatch>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackSearchMessageMatch {
    #[serde(rename = "iid")]
    pub id: String,
    pub channel: SlackSearchMessageMatchChannel,
    #[serde(default)]
    pub username: Option<String>,
    pub user: String,
    #[serde(default)]
    pub text: String,
    pub permalink: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub ts: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackSearchMessageMatchChannel {
    pub id: String,
    pub is_channel: bool,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackThread {
    pub messages: Vec<SlackThreadMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackThreadMessage {
    pub text: String,
    pub user: String,
    pub reply_count: Option<u32>,
}
