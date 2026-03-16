use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct EventsResponse {
    pub items: Option<Vec<EventItem>>,
}

#[derive(Deserialize)]
pub struct EventItem {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub start: EventTime,
    pub end: EventTime,
    pub status: Option<String>,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
    #[serde(rename = "colorId")]
    pub color_id: Option<String>,
    pub location: Option<String>,
    #[serde(rename = "hangoutLink")]
    pub hangout_link: Option<String>,
    #[serde(rename = "conferenceData")]
    pub conference_data: Option<ConferenceData>,
    pub organizer: Option<AttendeeItem>,
    pub attendees: Option<Vec<AttendeeItem>>,
    pub reminders: Option<RemindersItem>,
    #[serde(rename = "workingLocationProperties")]
    pub working_location_properties: Option<WorkingLocationProperties>,
}

#[derive(Deserialize)]
pub struct EventTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    pub date: Option<String>,
}

#[derive(Deserialize)]
pub struct ConferenceData {
    #[serde(rename = "entryPoints")]
    pub entry_points: Option<Vec<EntryPoint>>,
}

#[derive(Deserialize)]
pub struct EntryPoint {
    #[serde(rename = "entryPointType")]
    pub entry_point_type: String,
    pub uri: String,
}

#[derive(Deserialize)]
pub struct AttendeeItem {
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "responseStatus")]
    pub response_status: Option<String>,
    pub organizer: Option<bool>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
    #[allow(dead_code)]
    pub optional: Option<bool>,
}

#[derive(Deserialize)]
pub struct WorkingLocationProperties {
    #[serde(rename = "type")]
    pub location_type: Option<String>,
    // #[serde(rename = "homeOffice")]
    // #[allow(dead_code)]
    // pub home_office: Option<serde_json::Value>,
    #[serde(rename = "officeLocation")]
    pub office_location: Option<OfficeLocation>,
    #[serde(rename = "customLocation")]
    pub custom_location: Option<CustomLocation>,
}

#[derive(Deserialize)]
pub struct OfficeLocation {
    pub label: Option<String>,
    #[serde(rename = "buildingId")]
    pub building_id: Option<String>,
}

#[derive(Deserialize)]
pub struct CustomLocation {
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct RemindersItem {
    pub overrides: Option<Vec<ReminderOverride>>,
}

#[derive(Deserialize)]
pub struct ReminderOverride {
    pub method: String,
    pub minutes: i32,
}

#[derive(Deserialize)]
pub struct ColorsResponse {
    pub event: HashMap<String, ColorDef>,
}

#[derive(Deserialize)]
pub struct ColorDef {
    pub background: String,
}

pub const COLOR_NAME_MAP: &[(&str, &str)] = &[
    ("#a4bdfc", "Lavender"),
    ("#7ae7bf", "Sage"),
    ("#dbadff", "Grape"),
    ("#ff887c", "Flamingo"),
    ("#fbd75b", "Banana"),
    ("#ffb878", "Tangerine"),
    ("#46d6db", "Peacock"),
    ("#e1e1e1", "Graphite"),
    ("#5484ed", "Blueberry"),
    ("#51b749", "Basil"),
    ("#dc2127", "Tomato"),
];
