use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use reqwest::Client;
use std::collections::HashMap;

use crate::core::message::{Message, MessageStatus, MessageType, Person, PersonRole, Priority};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;
use crate::plugins::google::auth::{refresh_access_token, GoogleOAuthConfig};
use crate::plugins::google::calendar::model::*;

const CALENDAR_API_BASE: &str = "https://www.googleapis.com/calendar/v3";

pub struct GoogleCalendarClient {
    http: Client,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
    color_labels: HashMap<String, String>,
    upcoming_days: i64,
}

impl GoogleCalendarClient {
    pub fn new(
        config: &GoogleOAuthConfig,
        color_labels: HashMap<String, String>,
        upcoming_days: i64,
    ) -> Self {
        Self {
            http: Client::new(),
            access_token: config.access_token.clone(),
            refresh_token: config.refresh_token.clone(),
            expires_at: config.expires_at,
            color_labels,
            upcoming_days,
        }
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let token = self.fresh_token().await?;
        let url = format!("{}/calendars/primary", CALENDAR_API_BASE);
        println!("API call to Google Calendar: {}", &url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
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
        let color_map = self.fetch_color_map(&token).await;

        // Use the current date range from the sync command as the base,
        // then extend the end by upcoming_days so future events are always visible.
        let range = DateRange::get();
        let time_end = range.end + Duration::days(self.upcoming_days);

        let time_min = urlencoding::encode(&range.start.to_rfc3339()).into_owned();
        let time_max = urlencoding::encode(&time_end.to_rfc3339()).into_owned();

        let url = format!(
            "{}/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime",
            CALENDAR_API_BASE, time_min, time_max
        );
        println!("API call to Google Calendar: {}", &url);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| WorkOsError::Google(format!("Failed to fetch calendar events: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(WorkOsError::Google(format!(
                "Calendar API error {}: {}",
                status, body
            )));
        }

        let data: EventsResponse = resp.json().await.map_err(|e| {
            WorkOsError::Google(format!("Failed to parse calendar response: {}", e))
        })?;

        let messages = data
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| self.build_message(item, &color_map))
            .collect();

        Ok(messages)
    }

    // ============================
    // Google Calendar API
    // ============================

    async fn fetch_color_map(&self, token: &str) -> HashMap<String, String> {
        let url = format!("{}/colors", CALENDAR_API_BASE);
        println!("API call to Google Calendar: {}", &url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await;

        let Ok(resp) = resp else {
            return HashMap::new();
        };
        if !resp.status().is_success() {
            return HashMap::new();
        }

        let Ok(data) = resp.json::<ColorsResponse>().await else {
            return HashMap::new();
        };

        let hex_to_name: HashMap<&str, &str> = COLOR_NAME_MAP.iter().cloned().collect();
        data.event
            .into_iter()
            .filter_map(|(id, def)| {
                hex_to_name
                    .get(def.background.as_str())
                    .map(|name| (id, name.to_string()))
            })
            .collect()
    }

    // ============================
    // Helpers
    // ============================

    async fn fresh_token(&self) -> Result<String> {
        let needs_refresh = self
            .expires_at
            .map(|exp| exp < Utc::now().timestamp() + 300)
            .unwrap_or(false);

        if needs_refresh {
            if let Some(ref rt) = self.refresh_token {
                return refresh_access_token(rt).await;
            }
        }

        Ok(self.access_token.clone())
    }

    fn build_message(
        &self,
        item: EventItem,
        color_map: &HashMap<String, String>,
    ) -> Option<Message> {
        let (start, all_day) = parse_event_time(&item.start)?;
        let end = parse_event_time(&item.end).map(|(dt, _)| dt)?;

        if item.status.as_deref() == Some("cancelled") {
            return None;
        }

        let title = item.summary.unwrap_or_else(|| "(No title)".to_string());

        let now = Utc::now();
        let status = if end < now {
            MessageStatus::Done
        } else if start <= now {
            MessageStatus::InProgress
        } else {
            MessageStatus::Open
        };

        let event_type = item.event_type.as_deref().unwrap_or("default");

        // Resolve color: API color_id → Google name (e.g. "Sage") → user label (e.g. "Focus time")
        let google_color_name = item
            .color_id
            .as_ref()
            .and_then(|id| color_map.get(id))
            .map(String::as_str);

        let color_label = google_color_name
            .and_then(|name| self.color_labels.get(name))
            .map(String::as_str);

        let priority = map_priority(event_type);

        let meeting_link = item.hangout_link.or_else(|| {
            item.conference_data
                .and_then(|c| c.entry_points)
                .and_then(|eps| {
                    eps.into_iter()
                        .find(|e| e.entry_point_type == "video")
                        .map(|e| e.uri)
                })
        });

        let url = meeting_link.clone().unwrap_or_default();
        let description = build_description(
            &start,
            &end,
            all_day,
            event_type,
            google_color_name,
            color_label,
            item.location.as_deref(),
            item.working_location_properties.as_ref(),
            meeting_link.as_deref(),
            item.organizer.as_ref(),
            item.attendees.as_deref().unwrap_or(&[]),
            item.reminders.as_ref(),
            item.description.as_deref(),
        );

        let mut message = Message::new(
            "google_calendar",
            MessageType::CalendarEvent,
            &item.id,
            title,
            url,
        )
        .with_description(description)
        .with_priority(priority)
        .with_status(status)
        .with_date(start, start);

        if let Some(ref org) = item.organizer {
            let name = org
                .display_name
                .clone()
                .unwrap_or_else(|| org.email.clone());
            message = message.with_person(Person {
                name,
                username: org.email.clone(),
                role: PersonRole::Author,
            });
        }

        let attendees = item.attendees.as_deref().unwrap_or(&[]);
        if let Some(me) = attendees.iter().find(|a| a.is_self == Some(true)) {
            let name = me.display_name.clone().unwrap_or_else(|| me.email.clone());
            message = message.with_person(Person {
                name,
                username: me.email.clone(),
                role: PersonRole::Assignee,
            });
        }

        Some(message)
    }
}

// ============================
// Helpers
// ============================

fn parse_event_time(time: &EventTime) -> Option<(DateTime<Utc>, bool)> {
    if let Some(ref dt_str) = time.date_time {
        let dt = dt_str.parse::<DateTime<chrono::FixedOffset>>().ok()?;
        return Some((dt.to_utc(), false));
    }
    if let Some(ref date_str) = time.date {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
        let local_dt = Local
            .from_local_datetime(&naive.and_hms_opt(0, 0, 0)?)
            .earliest()?;
        return Some((local_dt.to_utc(), true));
    }
    None
}

fn map_priority(event_type: &str) -> Priority {
    match event_type {
        "focusTime" => Priority::High,
        _ => Priority::Unknown,
    }
}

fn build_description(
    start: &DateTime<Utc>,
    end: &DateTime<Utc>,
    all_day: bool,
    event_type: &str,
    google_color_name: Option<&str>,
    color_label: Option<&str>,
    location: Option<&str>,
    working_location: Option<&WorkingLocationProperties>,
    meeting_link: Option<&str>,
    organizer: Option<&AttendeeItem>,
    attendees: &[AttendeeItem],
    reminders: Option<&RemindersItem>,
    description_html: Option<&str>,
) -> String {
    let mut lines: Vec<String> = Vec::new();

    if all_day {
        lines.push("Time: All day".to_string());
    } else {
        let start_local = start.with_timezone(&Local);
        let end_local = end.with_timezone(&Local);
        let duration_mins = (*end - *start).num_minutes();
        let duration = if duration_mins < 60 {
            format!("{}m", duration_mins)
        } else if duration_mins % 60 == 0 {
            format!("{}h", duration_mins / 60)
        } else {
            format!("{}h {}m", duration_mins / 60, duration_mins % 60)
        };
        lines.push(format!(
            "Time: {} - {} ({})",
            start_local.format("%l:%M %p").to_string().trim(),
            end_local.format("%l:%M %p").to_string().trim(),
            duration
        ));
    }

    let type_label = match event_type {
        "focusTime" => "🟣 Focus Time",
        "outOfOffice" => "🏖️ Out of Office",
        "workingLocation" => "📍 Working Location",
        _ => "📅 Meeting",
    };
    lines.push(format!("Type: {}", type_label));

    let display_color = color_label.or(google_color_name);
    if let Some(color) = display_color {
        lines.push(format!("Label: {}", color));
    }

    if let Some(loc) = location {
        lines.push(format!("Location: {}", loc));
    }

    if let Some(wl) = working_location {
        let label = match wl.location_type.as_deref() {
            Some("homeOffice") => "Home".to_string(),
            Some("officeLocation") => wl
                .office_location
                .as_ref()
                .and_then(|o| o.label.as_deref().or(o.building_id.as_deref()))
                .unwrap_or("Office")
                .to_string(),
            Some("customLocation") => wl
                .custom_location
                .as_ref()
                .and_then(|c| c.label.as_deref())
                .unwrap_or("Custom")
                .to_string(),
            _ => "Unknown".to_string(),
        };
        lines.push(format!("Work Location: {}", label));
    }

    lines.push(String::new());

    if let Some(org) = organizer {
        let name = org.display_name.as_deref().unwrap_or(&org.email);
        lines.push(format!("Organizer: {} ({})", name, org.email));
    }

    let non_organizers: Vec<_> = attendees
        .iter()
        .filter(|a| a.organizer != Some(true))
        .collect();
    if !non_organizers.is_empty() {
        lines.push(format!("Attendees ({}):", non_organizers.len()));
        for att in non_organizers.iter().take(10) {
            let name = att.display_name.as_deref().unwrap_or(&att.email);
            let (icon, label) = match att.response_status.as_deref() {
                Some("accepted") => ("✓", "accepted"),
                Some("declined") => ("✗", "declined"),
                Some("tentative") => ("?", "tentative"),
                _ => ("•", "awaiting"),
            };
            let you = if att.is_self == Some(true) {
                " (You)"
            } else {
                ""
            };
            lines.push(format!("  {} {}{} ({})", icon, name, you, label));
        }
        if non_organizers.len() > 10 {
            lines.push(format!("  ... and {} more", non_organizers.len() - 10));
        }
    }

    if let Some(r) = reminders {
        if let Some(ref overrides) = r.overrides {
            if !overrides.is_empty() {
                lines.push(String::new());
                lines.push("Reminders:".to_string());
                for reminder in overrides {
                    lines.push(format!(
                        "  • {} ({})",
                        format_reminder_time(reminder.minutes),
                        reminder.method
                    ));
                }
            }
        }
    }

    if let Some(link) = meeting_link {
        lines.push(String::new());
        lines.push(format!("Meeting Link: {}", link));
    }

    if let Some(html) = description_html {
        let plain = htmd::convert(html).unwrap_or_else(|_| html.to_string());
        let plain = plain.trim();
        if !plain.is_empty() {
            lines.push(String::new());
            lines.push("Description:".to_string());
            let truncated = if plain.len() > 500 {
                format!("{}...", &plain[..500])
            } else {
                plain.to_string()
            };
            lines.push(truncated);
        }
    }

    lines.join("\n")
}

fn format_reminder_time(minutes: i32) -> String {
    if minutes < 60 {
        format!("{} min before", minutes)
    } else if minutes < 1440 {
        let h = minutes / 60;
        if h == 1 {
            "1 hour before".to_string()
        } else {
            format!("{} hours before", h)
        }
    } else {
        let d = minutes / 1440;
        if d == 1 {
            "1 day before".to_string()
        } else {
            format!("{} days before", d)
        }
    }
}
