use serde::{Deserialize};

#[derive(Debug, Deserialize)]
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
}

#[derive(Debug, Deserialize)]
pub struct AuthTestData {
    pub user_id: String,
    pub user: String,
    pub team: String,
}