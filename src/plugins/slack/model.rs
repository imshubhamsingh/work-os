use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub token: String,
    pub keywords: Vec<String>,
    pub channels: Vec<String>,
    pub output_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackResponse<T> {
    pub ok: bool,

    #[serde(default)]
    pub error: Option<String>,

    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationsInfoData {
    pub channel: SlackChannel,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SlackChannel {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_member: bool,
    // #[serde(default)]
    // pub is_group: bool,
    #[serde(default)]
    pub is_im: bool,
    #[serde(default)]
    pub is_mpim: bool,
    #[serde(default)]
    pub user: Option<String>,
    pub purpose: Option<SlackChannelPurpose>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SlackChannelPurpose {
    #[serde(default)]
    pub value: String,
    // #[serde(default)]
    // pub creator: String,
    // #[serde(default)]
    // pub last_set: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackMessage {
    // #[serde(rename = "type")]
    // pub message_type: String,
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
    #[serde(default)]
    pub attachments: Option<Vec<SlackMessageAttachment>>,
    #[serde(default)]
    pub saved: Option<SlackSaved>,
}

impl SlackMessage {
    pub fn get_forwarded_message(&self) -> Option<&SlackMessageAttachment> {
        self.attachments
            .as_ref()?
            .iter()
            .find(|att| att.is_forwarded_message())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackReaction {
    pub name: String,
    pub count: u32,
    #[serde(default)]
    pub users: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackSaved {
    pub is_archived: bool,
    pub date_completed: u64,
    pub date_due: u64,
    pub state: String,
}

impl SlackSaved {
    pub fn due_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        if self.date_due == 0 {
            return None;
        }
        chrono::DateTime::from_timestamp(self.date_due as i64, 0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackMessageAttachment {
    pub ts: Option<String>,
    pub channel_id: Option<String>,
    pub author_id: Option<String>,
    pub author_name: Option<String>,
    #[serde(default)]
    pub is_share: bool,
    #[serde(default)]
    pub is_msg_unfurl: bool,
    #[serde(default)]
    pub is_reply_unfurl: bool,
    pub from_url: Option<String>,
    pub text: Option<String>,
    pub fallback: Option<String>,
}

impl SlackMessageAttachment {
    pub fn is_forwarded_message(&self) -> bool {
        self.is_share && self.is_msg_unfurl && self.from_url.is_some()
    }
}

#[derive(Debug, Deserialize)]
pub struct ConversationsListData {
    pub channels: Vec<SlackChannel>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationsHistoryData {
    pub messages: Vec<SlackMessage>,
    // pub has_more: bool,
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

impl SlackUser {
    pub const UNKNOWN_ID: &'static str = "-1";

    pub fn unkown(user_id: &str) -> SlackUser {
        SlackUser {
            id: Self::UNKNOWN_ID.to_string(),
            name: format!("Unknown user {}", user_id),
            real_name: None,
            deleted: false,
            is_bot: false,
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.id == Self::UNKNOWN_ID
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackUserGroup {
    pub id: String,
    pub handle: String,
}

impl SlackUserGroup {
    pub const UNKNOWN_ID: &'static str = "-1";

    pub fn unknown(group_id: &str) -> SlackUserGroup {
        SlackUserGroup {
            id: Self::UNKNOWN_ID.to_string(),
            handle: format!("unknown-group-{}", group_id),
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.id == Self::UNKNOWN_ID
    }
}

#[derive(Debug, Deserialize)]
pub struct UserGroupsListData {
    pub usergroups: Vec<SlackUserGroup>,
}

#[derive(Debug, Deserialize)]
pub struct AuthTestData {
    pub user_id: String,
    // pub user: String,
    // pub team: String,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackThread {
    pub messages: Vec<SlackThreadMessage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackThreadMessage {
    pub text: String,
    pub user: String,
    pub ts: String,
    pub reply_count: Option<u32>,
    #[serde(default)]
    pub thread_ts: Option<String>,
    #[serde(default)]
    pub reactions: Option<Vec<SlackReaction>>,
    #[serde(default)]
    pub attachments: Option<Vec<SlackMessageAttachment>>,
    #[serde(default)]
    pub saved: Option<SlackSaved>,
}

impl SlackThreadMessage {
    pub fn get_forwarded_message(&self) -> Option<&SlackMessageAttachment> {
        self.attachments
            .as_ref()?
            .iter()
            .find(|att| att.is_forwarded_message())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilesListData {
    pub files: Vec<SlackCanvas>,
    pub paging: FilesListPaging,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilesListPaging {
    pub pages: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackCanvas {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub created: u64,
    #[serde(default)]
    pub updated: u64,
    pub permalink: Option<String>,
    pub url_private_download: Option<String>,
    #[serde(default)]
    pub editors: Vec<String>,
    #[serde(default)]
    pub channels: Vec<String>,
}

impl SlackCanvas {
    pub fn url(&self) -> Option<&str> {
        self.permalink.as_deref()
    }

    pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(self.updated as i64, 0).unwrap_or_else(chrono::Utc::now)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilesInfoData {
    pub file: SlackFileInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackFileInfo {
    pub shares: Option<SlackFileShares>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SlackFileShares {
    #[serde(default)]
    pub public: HashMap<String, Vec<SlackFileShare>>,
    #[serde(default)]
    pub private: HashMap<String, Vec<SlackFileShare>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackFileShare {
    pub ts: String,
    #[serde(default)]
    pub reply_count: u64,
}
