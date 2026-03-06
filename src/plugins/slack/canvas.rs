use chrono::{DateTime, Local, Utc};

use crate::core::message::{Message, MessageType};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;

use super::client::SlackClient;
use super::model::*;

pub async fn fetch_all_canvases(
    client: &SlackClient,
    user_id: &str,
) -> Result<(Vec<SlackCanvas>, Vec<SlackCanvas>)> {
    let date_range = DateRange::get();
    let oldest = date_range.start.timestamp() as u64;
    let newest = date_range.end.timestamp() as u64;

    let mut directly_involved = Vec::new();
    let mut others = Vec::new();
    let mut current_page = 1u32;

    loop {
        let response: SlackResponse<FilesListData> = client
            .get(&format!(
                "files.list?types=canvas&count=2000&page={}",
                current_page
            ))
            .await?;

        if !response.ok {
            return Err(WorkOsError::Slack(
                response
                    .error
                    .unwrap_or_else(|| "unknown error".to_string()),
            ));
        }

        let Some(data) = response.data else { break };
        let total_pages = data.paging.pages;

        for canvas in data.files {
            let created_in_range = canvas.created >= oldest && canvas.created <= newest;
            let updated_in_range = canvas.updated >= oldest && canvas.updated <= newest;
            if !created_in_range && !updated_in_range {
                continue;
            }

            let is_directly_involved = canvas.editors.iter().any(|e| e == user_id)
                || canvas
                    .channels
                    .iter()
                    .any(|ch| client.channels.contains(ch));

            if is_directly_involved {
                directly_involved.push(canvas);
            } else {
                others.push(canvas);
            }
        }

        if current_page >= total_pages {
            break;
        }
        current_page += 1;
    }
    Ok((directly_involved, others))
}

pub async fn process_involved_canvas(
    client: &mut SlackClient,
    canvas: &SlackCanvas,
    user_id: &str,
    user_mention: &str,
) -> Result<Option<Message>> {
    let Some(slack_url) = canvas.url() else {
        return Ok(None);
    };
    let slack_url = slack_url.to_string();
    let updated_at = canvas.updated_at();

    let channel_messages = get_canvas_channel_messages(client, &canvas.id)
        .await
        .unwrap_or_default();

    let Some(download_url) = &canvas.url_private_download else {
        let msg =
            build_canvas_message(canvas, slack_url, updated_at, &channel_messages, false, &[]);
        return Ok(Some(msg));
    };

    let raw_content = fetch_canvas_content(client, download_url)
        .await
        .map_err(|e| WorkOsError::Slack(format!("Failed to download '{}': {}", canvas.title, e)))?;

    let is_mentioned = raw_content.contains(user_mention);
    let comment_mentions = fetch_canvas_share_mentions(client, &canvas.id, user_id)
        .await
        .unwrap_or_default();
    let has_mention = is_mentioned || !comment_mentions.is_empty();

    write_canvas_file(
        client,
        canvas,
        &slack_url,
        &raw_content,
        is_mentioned,
        &comment_mentions,
    );

    Ok(Some(build_canvas_message(
        canvas,
        slack_url,
        updated_at,
        &channel_messages,
        has_mention,
        &comment_mentions,
    )))
}

// Observer only included when @mention found in the any canvas channel. Downloads canvas file if so.
pub async fn process_observer_canvas(
    client: &mut SlackClient,
    canvas: &SlackCanvas,
    user_mention: &str,
) -> Result<Option<Message>> {
    let Some(slack_url) = canvas.url() else {
        return Ok(None);
    };
    let slack_url = slack_url.to_string();
    let updated_at = canvas.updated_at();

    let channel_messages = get_canvas_channel_messages(client, &canvas.id)
        .await
        .unwrap_or_default();
    if !channel_messages.contains(user_mention) {
        return Ok(None);
    }

    if let Some(download_url) = &canvas.url_private_download {
        let raw_content = fetch_canvas_content(client, download_url)
            .await
            .map_err(|e| {
                WorkOsError::Slack(format!("Failed to download '{}': {}", canvas.title, e))
            })?;
        write_canvas_file(client, canvas, &slack_url, &raw_content, false, &[]);
    }

    Ok(Some(build_canvas_message(
        canvas,
        slack_url,
        updated_at,
        &channel_messages,
        true,
        &[],
    )))
}

fn write_canvas_file(
    client: &SlackClient,
    canvas: &SlackCanvas,
    slack_url: &str,
    raw_content: &str,
    is_mentioned: bool,
    comment_mentions: &[String],
) {
    match client.canvas_writer.write_canvas(
        &canvas.id,
        &canvas.title,
        slack_url,
        canvas.updated,
        raw_content,
        is_mentioned,
        comment_mentions,
    ) {
        Ok(Some((path, _))) => println!("  ✓ Wrote '{}': {}", canvas.title, path.display()),
        Ok(None) => {}
        Err(e) => eprintln!("  ✗ Write failed '{}': {}", canvas.title, e),
    }
}

async fn fetch_canvas_content(client: &SlackClient, url: &str) -> Result<String> {
    client
        .http
        .get(url)
        .header("Authorization", format!("Bearer {}", client.token))
        .send()
        .await
        .map_err(|e| WorkOsError::Slack(format!("Failed to download canvas: {}", e)))?
        .text()
        .await
        .map_err(|e| WorkOsError::Slack(format!("Failed to read canvas: {}", e)))
}

// Thread replies from channels where this canvas was shared that @mention the user.
async fn fetch_canvas_share_mentions(
    client: &mut SlackClient,
    canvas_id: &str,
    user_id: &str,
) -> Result<Vec<String>> {
    let response: SlackResponse<FilesInfoData> = client
        .get(&format!("files.info?file={}", canvas_id))
        .await?;
    if !response.ok {
        return Ok(Vec::new());
    }

    let shares = match response.data.and_then(|d| d.file.shares) {
        Some(s) => s,
        None => return Ok(Vec::new()),
    };

    let user_mention = format!("<@{}>", user_id);
    let mut mentions = Vec::new();

    let all_shares = shares.public.into_iter().chain(shares.private);
    for (channel_id, share_list) in all_shares {
        for share in share_list.into_iter().filter(|s| s.reply_count > 0) {
            let mentioned_texts: Vec<String> = client
                .get_thread_messages(&channel_id, &share.ts)
                .await?
                .into_iter()
                .filter(|msg| msg.text.contains(&user_mention))
                .map(|msg| msg.text)
                .collect();

            for text in mentioned_texts {
                mentions.push(client.replace_user_id_with_handle(&text).await?);
            }
        }
    }
    Ok(mentions)
}

// All messages in the canvas backing channel (F→C) within the date range, with thread replies.
async fn get_canvas_channel_messages(client: &mut SlackClient, canvas_id: &str) -> Result<String> {
    let Some(channel_id) = canvas_id.strip_prefix('F').map(|rest| format!("C{}", rest)) else {
        return Ok(String::new());
    };

    let messages = client.get_channel_messages(&channel_id).await?;
    if messages.is_empty() {
        return Ok(String::new());
    }

    client
        .build_description_from_message_and_thread(&channel_id, &messages)
        .await
}

fn build_canvas_message(
    canvas: &SlackCanvas,
    slack_url: String,
    updated_at: DateTime<Utc>,
    channel_messages: &str,
    has_mention: bool,
    comment_mentions: &[String],
) -> Message {
    let mut parts = vec![format!(
        "Updated: {}",
        updated_at.with_timezone(&Local).format("%b %d, %l:%M %p")
    )];

    if has_mention {
        parts.push("**You are mentioned in this canvas.**".to_string());
    }
    if !comment_mentions.is_empty() {
        parts.push(format!(
            "Mentioned in {} comment(s):",
            comment_mentions.len()
        ));
        parts.extend(comment_mentions.iter().map(|c| format!("  > {}", c)));
    }
    if !channel_messages.is_empty() {
        parts.push(format!("\n## Canvas comments\n{}", channel_messages));
    }

    let title = if has_mention {
        format!("Canvas: {} [Mentioned]", canvas.title)
    } else {
        format!("Canvas: {}", canvas.title)
    };

    Message::new("slack", MessageType::Canvas, &canvas.id, title, slack_url)
        .with_date(updated_at, updated_at)
        .with_description(parts.join("\n"))
}
