//! Small English formatting helpers (chrono has no stable locale data).

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

pub fn date_en(date: NaiveDate) -> String {
    let month = MONTHS[(date.month0() as usize).min(11)];
    format!("{} {}, {}", month, date.day(), date.year())
}

pub fn time_en(value: DateTime<Utc>) -> String {
    value.with_timezone(&Local).format("%H:%M").to_string()
}
