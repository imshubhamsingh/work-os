use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Utc,
    Weekday,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::{error::WorkOsError, models::state::WorkOsState};

static DATE_RANGE: OnceLock<DateRange> = OnceLock::new();

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RunMode {
    Today,
    SinceLastRun,
    Weekend,
    Days(u32),
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub mode: RunMode,
}

impl DateRange {
    pub fn today() -> Self {
        let now = Utc::now();
        let start = Local::now()
            .date_naive()
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_utc();
        Self {
            start,
            end: now,
            mode: RunMode::Today,
        }
    }

    pub fn since(last_run: DateTime<Utc>) -> Self {
        Self {
            start: last_run,
            end: Utc::now(),
            mode: RunMode::SinceLastRun,
        }
    }

    pub fn weekend() -> Self {
        let today = Local::now().date_naive();

        let days_since_friday = match today.weekday() {
            Weekday::Mon => 3,
            Weekday::Tue => 4,
            Weekday::Wed => 5,
            Weekday::Thu => 6,
            Weekday::Fri => 0,
            Weekday::Sat => 1,
            Weekday::Sun => 2,
        };

        let friday = today - TimeDelta::days(days_since_friday);
        let start = friday
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_utc();

        Self {
            start,
            end: Utc::now(),
            mode: RunMode::Weekend,
        }
    }

    pub fn last_n_days(days: u32) -> Self {
        let end = Utc::now();
        let start = end - TimeDelta::days(days as i64);
        Self {
            start,
            end: Utc::now(),
            mode: RunMode::Days(days),
        }
    }

    pub fn custom(from: &str, to: Option<&str>) -> Result<Self, String> {
        let start = parse_local_datetime(from, NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .map_err(|e| format!("Invalid 'from' date: {}", e))?;

        let end = match to {
            Some(to_str) => {
                parse_local_datetime(to_str, NaiveTime::from_hms_opt(23, 59, 59).unwrap())
                    .map_err(|e| format!("Invalid 'to' date: {}", e))?
            }
            None => Utc::now(),
        };

        if end < start {
            return Err(format!("'to' date must be after 'from' date"));
        }

        Ok(Self {
            start,
            end,
            mode: RunMode::Custom,
        })
    }

    pub fn describe(&self) -> String {
        match self.mode {
            RunMode::Today => "today".to_string(),
            RunMode::SinceLastRun => format!(
                "since {}",
                self.start.with_timezone(&Local).format("%Y-%m-%d %H:%M")
            ),
            RunMode::Weekend => "weekend (Fri-Sun)".to_string(),
            RunMode::Days(n) => format!("last {} days", n),
            RunMode::Custom => format!(
                "{} to {}",
                self.start.format("%Y-%m-%d"),
                self.end.format("%Y-%m-%d")
            ),
        }
    }

    pub fn init(range: DateRange) {
        DATE_RANGE
            .set(range)
            .expect("DateRange already initialized");
    }

    pub fn get() -> &'static DateRange {
        DATE_RANGE
            .get()
            .expect("DateRange not initialized - call DateRange::init() first")
    }

    pub fn contains(&self, dt: DateTime<Utc>) -> bool {
        dt >= self.start && dt <= self.end
    }

    pub fn resolve_date_range(
        from: Option<&str>,
        to: Option<&str>,
        run_mode: &str,
        state: &WorkOsState,
    ) -> Result<DateRange, WorkOsError> {
        if from.is_some() || to.is_some() {
            if to.is_some() && from.is_none() {
                return Err(WorkOsError::Config(
                    "--to requires --from to also be specified".to_string(),
                ));
            }
            return DateRange::custom(from.unwrap(), to).map_err(WorkOsError::Config);
        }

        match run_mode {
            "weekend" => Ok(DateRange::weekend()),

            "since-last-run" => {
                let last_run = state
                    .daily_brief
                    .last_run
                    .unwrap_or_else(|| Utc::now() - TimeDelta::days(1));

                Ok(DateRange::since(last_run))
            }

            mode if mode.starts_with("days-") => {
                let days = parse_days(mode)?;
                Ok(DateRange::last_n_days(days))
            }

            _ => Ok(DateRange::today()),
        }
    }
}

impl std::fmt::Display for DateRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.start.format("%Y-%m-%d %H:%M"),
            self.end.format("%Y-%m-%d %H:%M"),
        )
    }
}

fn parse_local_datetime(s: &str, default_time: NaiveTime) -> Result<DateTime<Utc>, String> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d").map(|d| d.and_time(default_time)))
        .map_err(|_| "expected format YYYY-MM-DD or YYYY-MM-DD HH:MM".to_string())?;

    Local
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.to_utc())
        .ok_or_else(|| format!("ambiguous or invalid local time: {}", s))
}

fn parse_days(mode: &str) -> Result<u32, WorkOsError> {
    mode.strip_prefix("days-")
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|&n| n > 0)
        .ok_or_else(|| {
            WorkOsError::Config(format!(
                "Invalid run mode `{}`. Expected format: days-N (N > 0)",
                mode
            ))
        })
}
