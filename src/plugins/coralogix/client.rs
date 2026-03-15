use std::collections::{HashMap, HashSet};
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::{fs, time};

use chrono::{DateTime, Local, Utc};
use reqwest::Client;
use serde_json::Value;

use crate::core::message::{Message, MessageType, Priority};
use crate::error::{Result, WorkOsError};
use crate::models::date_range::DateRange;
use crate::plugins::coralogix::config::CoralogixConfig;
use crate::plugins::coralogix::model::{CoralogixLog, Severity};

const DATAPRIME_ENDPOINT: &str = "https://api.eu1.coralogix.com/api/v1/dataprime/query";

pub struct CoralogixClient {
    http: Client,
    config: CoralogixConfig,
}

impl CoralogixClient {
    pub fn new(config: &CoralogixConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(time::Duration::from_secs(120))
            .build()
            .map_err(|e| WorkOsError::Coralogix(e.to_string()))?;
        Ok(Self {
            http,
            config: config.clone(),
        })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        Ok(!self.config.api_key.is_empty())
    }

    pub async fn get_all_messages(&self) -> Result<Vec<Message>> {
        let date_range = DateRange::get();

        println!(
            "  Querying Coralogix: [{}] errors, {} → {}",
            self.config.application_names.join(", "),
            date_range
                .start
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M"),
            date_range.end.with_timezone(&Local).format("%H:%M"),
        );

        let raw_records = self.query(date_range.start, date_range.end).await?;
        println!("  {} raw records fetched", raw_records.len());

        let today_date = date_range
            .end
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string();
        let (all_records, written, skipped) = self.write_jsonl(&raw_records, &today_date)?;

        println!("  {written} new records written, {skipped} dupes skipped");

        Ok(self.build_messages(&all_records, &today_date))
    }

    async fn query(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<CoralogixLog>> {
        let conditions: Vec<String> = self
            .config
            .application_names
            .iter()
            .map(|name| format!("$l.applicationname == '{name}'"))
            .collect();

        let app_filter = match conditions.as_slice() {
            [single] => single.clone(),
            _ => format!("({})", conditions.join(" || ")),
        };

        let query = format!(
            "source logs | filter $m.severity == ERROR | filter {app_filter} | filter $d.logRecord.attributes.environment == 'production' | orderby $m.timestamp desc"
        );

        let body = serde_json::json!({
            "query": query,
            "metadata": {
                "startDate": start.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                "endDate": end.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                "defaultSource": "logs",
                "syntax": "QUERY_SYNTAX_DATAPRIME",
            }
        });

        println!(
            "API call to Coralogix: {} | query: {} | from: {} to: {}",
            DATAPRIME_ENDPOINT,
            query,
            start.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            end.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
        );
        let response = self
            .http
            .post(DATAPRIME_ENDPOINT)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| WorkOsError::Coralogix(format!("Coralogix request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WorkOsError::Coralogix(format!(
                "Coralogix API {status}: {}",
                &text[..text.len().min(400)]
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| WorkOsError::Coralogix(e.to_string()))?;

        let mut records = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(chunk) = serde_json::from_str::<Value>(trimmed) {
                if let Some(results) = chunk
                    .get("result")
                    .and_then(|r| r.get("results"))
                    .and_then(|r| r.as_array())
                {
                    for r in results {
                        if let Some(record) = parse_record(r, &self.config.domain) {
                            records.push(record);
                        }
                    }
                }
            }
        }

        Ok(records)
    }

    fn jsonl_path(&self, date: &str) -> PathBuf {
        self.config.output_path.join(date).join("coralogix.jsonl")
    }

    fn write_jsonl(
        &self,
        records: &[CoralogixLog],
        date: &str,
    ) -> Result<(Vec<CoralogixLog>, usize, usize)> {
        let path = self.jsonl_path(date);
        fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| WorkOsError::Coralogix(e.to_string()))?;

        let existing: Vec<CoralogixLog> = fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str(l.trim()).ok())
            .collect();

        let seen: HashSet<String> = existing.iter().map(|r| r.logid.clone()).collect();
        let mut all_records = existing;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| WorkOsError::Coralogix(e.to_string()))?;

        let (mut written, mut skipped) = (0, 0);
        for record in records {
            if seen.contains(&record.logid) {
                skipped += 1;
                continue;
            }
            writeln!(
                file,
                "{}",
                serde_json::to_string(record).map_err(|e| WorkOsError::Coralogix(e.to_string()))?
            )
            .map_err(|e| WorkOsError::Coralogix(e.to_string()))?;
            all_records.push(record.clone());
            written += 1;
        }

        Ok((all_records, written, skipped))
    }

    fn find_prev_records(&self, today_date: &str) -> (String, Vec<CoralogixLog>) {
        let mut dates: Vec<String> = fs::read_dir(&self.config.output_path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|d| is_date_folder(d) && d.as_str() < today_date)
            .collect();
        dates.sort_by(|a, b| b.cmp(a));

        for date in dates {
            let path = self.jsonl_path(&date);
            if path.exists() {
                let records = fs::read_to_string(&path)
                    .unwrap_or_default()
                    .lines()
                    .filter_map(|l| serde_json::from_str(l.trim()).ok())
                    .collect();
                return (date, records);
            }
        }
        (String::new(), Vec::new())
    }

    fn group_by_body(records: &[&CoralogixLog]) -> Vec<(String, usize, CoralogixLog)> {
        let mut freq: HashMap<&str, (usize, &CoralogixLog)> = HashMap::new();
        for r in records {
            let entry = freq.entry(r.body.as_str()).or_insert((0, r));
            entry.0 += 1;
            if r.timestamp > entry.1.timestamp {
                entry.1 = r;
            }
        }
        let mut sorted: Vec<(String, usize, CoralogixLog)> = freq
            .into_iter()
            .map(|(body, (count, rec))| (body.to_string(), count, rec.clone()))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted
    }

    fn build_analysis(
        today: &[(String, usize, CoralogixLog)],
        prev: &[(String, usize, CoralogixLog)],
        prev_date: &str,
        app_names: &[String],
    ) -> String {
        let total: usize = today.iter().map(|(_, count, _)| count).sum();
        let prev_total: usize = prev.iter().map(|(_, count, _)| count).sum();
        let has_prev = prev_total > 0;
        let prev_map: HashMap<&str, usize> = prev
            .iter()
            .map(|(body, count, _)| (body.as_str(), *count))
            .collect();

        let trend = if !has_prev {
            String::new()
        } else if total < prev_total {
            format!(" ↓ from {} prev", prev_total)
        } else if total > prev_total {
            format!(" ↑ from {} prev", prev_total)
        } else {
            format!(" = {} prev", prev_total)
        };

        let vs = match has_prev {
            true => format!(" (vs {})", prev_date),
            false => String::new(),
        };

        let mut out = format!(
            "## 🚨 Production Errors · {} ({} errors{}{})\n\n",
            app_names.join(", "),
            total,
            trend,
            vs
        );

        let recurring: Vec<_> = today.iter().filter(|(_, count, _)| *count >= 5).collect();
        let oneoffs: Vec<_> = today.iter().filter(|(_, count, _)| *count < 5).collect();

        if !recurring.is_empty() {
            out.push_str("### 🔁 Recurring — needs attention\n\n");
            let trend_col = if has_prev { " Trend |" } else { "" };
            out.push_str(&format!("| Count |{trend_col} Error | Link |\n"));
            out.push_str(&format!(
                "|-------|{}------|------|\n",
                if has_prev { "-------|" } else { "" }
            ));
            for (body, count, rec) in &recurring {
                let trend_cell = if has_prev {
                    match prev_map.get(body.as_str()) {
                        None => " 🆕 |".to_string(),
                        Some(&p) if *count > p => format!(" ↑ {}→{} |", p, count),
                        Some(&p) if *count < p => format!(" ↓ {}→{} |", p, count),
                        Some(&p) => format!(" = {} |", p),
                    }
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "| {}x |{} {} | [→]({}) |\n",
                    count, trend_cell, body, rec.url
                ));
            }
            out.push('\n');
        }

        if !oneoffs.is_empty() {
            out.push_str("### ⚠️ One-off Concerns\n\n");
            for (body, count, rec) in &oneoffs {
                let err = if rec.error.is_empty() {
                    String::new()
                } else {
                    format!(" — `{}`", &rec.error[..rec.error.len().min(80)])
                };
                let trend_tag = if !has_prev {
                    String::new()
                } else {
                    match prev_map.get(body.as_str()) {
                        None => " 🆕".to_string(),
                        Some(&p) => format!(" (was {}x prev)", p),
                    }
                };
                out.push_str(&format!(
                    "- `{}` ({}x{}){} — [→]({})\n",
                    body, count, trend_tag, err, rec.url
                ));
            }
            out.push('\n');
        }

        if has_prev {
            let new_errors: Vec<_> = today
                .iter()
                .filter(|(body, count, _)| *count >= 2 && !prev_map.contains_key(body.as_str()))
                .collect();
            if !new_errors.is_empty() {
                out.push_str(&format!("### 🆕 New Since Last Run ({})\n\n", prev_date));
                for (body, count, rec) in &new_errors {
                    let err = if rec.error.is_empty() {
                        String::new()
                    } else {
                        format!(" — `{}`", &rec.error[..rec.error.len().min(60)])
                    };
                    out.push_str(&format!(
                        "- `{}` ({}x){} — [→]({})\n",
                        body, count, err, rec.url
                    ));
                }
                out.push('\n');
            }

            let resolved: Vec<_> = prev
                .iter()
                .filter(|(body, count, _)| *count >= 5 && !today.iter().any(|(b, _, _)| b == body))
                .collect();
            if !resolved.is_empty() {
                out.push_str("### ✅ Resolved Since Last Run\n\n");
                for (body, count, _) in &resolved {
                    out.push_str(&format!("- `{}` — was {}x, gone today\n", body, count));
                }
                out.push('\n');
            }
        }

        out
    }

    fn build_messages(&self, records: &[CoralogixLog], today_date: &str) -> Vec<Message> {
        if records.is_empty() {
            return Vec::new();
        }

        let (prev_date, prev_records) = self.find_prev_records(today_date);

        let mut by_app: HashMap<String, Vec<&CoralogixLog>> = HashMap::new();
        for r in records {
            by_app
                .entry(r.application_name.clone())
                .or_default()
                .push(r);
        }

        let mut prev_by_app: HashMap<String, Vec<&CoralogixLog>> = HashMap::new();
        for r in &prev_records {
            prev_by_app
                .entry(r.application_name.clone())
                .or_default()
                .push(r);
        }

        by_app
            .iter()
            .map(|(app, today_app)| {
                let prev_app = prev_by_app.get(app).map(Vec::as_slice).unwrap_or(&[]);
                let today_grouped = Self::group_by_body(today_app);
                let prev_grouped = Self::group_by_body(prev_app);

                let desc =
                    Self::build_analysis(&today_grouped, &prev_grouped, &prev_date, &[app.clone()]);

                Message::new(
                    "coralogix",
                    MessageType::Coralogix,
                    &format!("summary-{}-{}", app, today_date),
                    format!(
                        "Production Errors: {} total ({} unique) — {}",
                        today_app.len(),
                        today_grouped.len(),
                        app
                    ),
                    format!("{}/#/query-new/logs", self.config.domain),
                )
                .with_description(desc)
                .with_priority(match today_app.len() > 100 {
                    true => Priority::High,
                    false => Priority::Medium,
                })
            })
            .collect()
    }
}

fn is_date_folder(name: &str) -> bool {
    name.len() == 10
        && name.bytes().enumerate().all(|(i, b)| match i {
            4 | 7 => b == b'-',
            _ => b.is_ascii_digit(),
        })
}

fn parse_record(r: &Value, domain: &str) -> Option<CoralogixLog> {
    let mut logid: Option<&str> = None;
    let mut timestamp: Option<&str> = None;
    let mut severity: Option<&str> = None;

    for item in r.get("metadata")?.as_array()? {
        match item.get("key").and_then(Value::as_str) {
            Some("logid") => logid = item.get("value").and_then(Value::as_str),
            Some("timestamp") => timestamp = item.get("value").and_then(Value::as_str),
            Some("severity") => severity = item.get("value").and_then(Value::as_str),
            _ => {}
        }
        if logid.is_some() && timestamp.is_some() && severity.is_some() {
            break;
        }
    }

    let application_name = r
        .get("labels")
        .and_then(|l| l.as_array())
        .and_then(|items| {
            items.iter().find_map(|item| {
                if item.get("key").and_then(Value::as_str) == Some("applicationname") {
                    item.get("value").and_then(Value::as_str)
                } else {
                    None
                }
            })
        })
        .unwrap_or("")
        .to_string();

    let logid = logid.filter(|s| !s.is_empty())?.to_string();
    let timestamp = timestamp.unwrap_or("").to_string();
    let severity = Severity::from_num(severity.and_then(|s| s.parse().ok()).unwrap_or(5));

    let ud: Value = match r.get("userData") {
        Some(Value::String(s)) => serde_json::from_str(s).unwrap_or(Value::Null),
        Some(v) => v.clone(),
        None => Value::Null,
    };

    let log_record = ud.get("logRecord").unwrap_or(&Value::Null);
    let attrs = log_record.get("attributes").unwrap_or(&Value::Null);
    let attr = |k: &str| {
        attrs
            .get(k)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let attr_or = |k: &str, default: &'static str| {
        attrs
            .get(k)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(default)
            .to_string()
    };

    let ts_ms = timestamp_to_ms(&timestamp);

    Some(CoralogixLog {
        url: format!(
            "{}/#/query-new/logs?permalink=true&startTime={}&endTime={}&logId={}",
            domain,
            ts_ms - 1000,
            ts_ms + 1000,
            logid
        ),
        logid,
        timestamp,
        application_name,
        severity,
        body: log_record
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        error: attr("error"),
        service: attr_or("service", "unkown-service"),
        trace_id: log_record
            .get("traceId")
            .and_then(Value::as_str)
            .or_else(|| attrs.get("dd.trace_id").and_then(Value::as_str))
            .unwrap_or("")
            .to_string(),
        span_id: attr("dd.span_id"),
        environment: attr_or("environment", "production"),
    })
}

fn parse_timestamp(timestamp: &str) -> DateTime<Utc> {
    if timestamp.is_empty() {
        return Utc::now();
    }
    let ts = if timestamp.ends_with('Z') {
        timestamp.to_string()
    } else {
        format!("{}Z", timestamp)
    };
    DateTime::parse_from_rfc3339(&ts)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

pub fn timestamp_to_ms(timestamp: &str) -> i64 {
    parse_timestamp(timestamp).timestamp_millis()
}
