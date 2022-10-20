use chrono::{NaiveDate, Duration, Datelike};
use chrono::offset::Local;

fn today() -> NaiveDate {
    Local::today().naive_local()
}

fn days_from_today(days: i64) -> NaiveDate {
    today().checked_sub_signed(Duration::days(days)).unwrap()
}

pub fn input(s: &str) -> Result<NaiveDate, String> {
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date)
    }
    match s {
        "today" => Ok(today()),
        "yesterday" => Ok(days_from_today(1)),
        "month" => {
            let current_day: i64 = today().day().into();
            Ok(days_from_today(current_day - 1))
        },
        _  => Err(format!("Can't format input as date: {}", s))
    }
}
