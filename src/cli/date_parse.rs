use chrono::{NaiveDate, Duration, Datelike, Weekday};
use chrono::offset::Local;

fn today() -> NaiveDate {
    Local::today().naive_local()
}

fn days_from_today(days: i64) -> NaiveDate {
    today().checked_sub_signed(Duration::days(days)).unwrap()
}

fn current_day() -> i64 {
    today().day().into()
}

pub fn input(s: &str) -> Result<NaiveDate, String> {
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date)
    }
    match s {
        "today" => Ok(today()),
        "yesterday" => Ok(days_from_today(1)),
        "month" => Ok(days_from_today(current_day() - 1)),
        "biweekly" => {
            let today = current_day();
            let target = if today >= 15 { 
                15
            } else {
                1
            };
            Ok(days_from_today(today - target))
        }
        "friday" => {
            let mut cursor = today();
            let mut last_friday_gap = 0;
            while cursor.weekday() != Weekday::Fri {
                cursor = cursor.checked_sub_signed(Duration::days(1)).unwrap();
                last_friday_gap += 1;
            };
            Ok(days_from_today(last_friday_gap))
        }
        _  => Err(format!("Can't format input as date: {}", s))
    }
}
