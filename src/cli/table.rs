use std::{collections::HashMap, ops::Range};

use lazy_static::lazy_static;
use regex::Regex;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use super::data::model::DiaryDoc;
use tabled::{builder::Builder, Style, Modify, object::Cell, Border};

struct PrintWithDatesData {
    dates: HashMap<NaiveDate, Vec<Vec<String>>>,
    durations: HashMap<NaiveDate, Duration>,
    total: Duration
}

impl PrintWithDatesData {
    pub fn new(results: Vec<DiaryDoc>) -> PrintWithDatesData {
        let mut inst = PrintWithDatesData {
            dates: HashMap::new(),
            durations: HashMap::new(),
            total: Duration::seconds(0)
        };
        let zero = Duration::seconds(0);
        for doc in results.iter() {
            for range in doc.worklog_range().iter() {
                let mut empty = vec!();
                let key = range.start.date();
                let value: &mut Vec<Vec<String>> = inst.dates
                    .get_mut(&key)
                    .unwrap_or(&mut empty);

                let (partial, row) = PrintWithDatesData::doc_row(fname(doc), range);
                value.push(row);

                let final_value = value.to_vec();
                inst.dates.insert(key.clone(), final_value);

                let this_date_duration = inst.durations.get(&key).unwrap_or(&zero);
                inst.durations.insert(key, *this_date_duration + partial);
                inst.total  = inst.total + partial;
            }
        }
        inst
    }

    fn doc_row(ticket: String, range: &Range<NaiveDateTime>) -> (Duration, Vec<String>) {
        let partial = range.end - range.start;
        (
            partial,
            vec![
                ticket,
                range.start.format("%H:%M").to_string(),
                range.end.format("%H:%M").to_string(),
                duration_to_string(&partial),
                String::from("")
            ]
        )
    }

}

fn duration_to_string(duration: &Duration) -> String {
    String::from(
        format!(
            "{}h {:02}m", 
            duration.num_hours(), 
            duration.num_minutes() - duration.num_hours() * 60
        )
    )
}

fn fname(doc: &DiaryDoc) -> String {
    String::from(doc.path.split("/").last().unwrap())
}

// This method will return:
// - Rows from doc's worklogs
// - Accumulated duration from all those rows
fn ranges(doc: &DiaryDoc) -> (String, Duration) {
    let mut duration_acc = Duration::seconds(0);
    let rows = doc.worklog_range()
        .iter()
        .map(|y| {
            let diff = y.end - y.start;
            let dur = duration_to_string(&(y.end - y.start));
            duration_acc = duration_acc + diff;
            format!("{} -> {}: {}", y.start, y.end, dur)
        })
        .collect::<Vec<String>>()
        .join("\n");
    (rows, duration_acc)
}

fn fmt_total(duration: Duration) -> Vec<String> {
    vec!(
        "".to_string(), 
        "".to_string(), 
        duration_to_string(&duration)
    )
}

fn do_print(builder: Builder) {
    let mut table = builder.build();
    table.with(Style::psql());

    let last_border = Border::empty()
        .top('-')
        .top_left_corner('+');
    let selector = Cell(table.count_rows() - 1, table.count_columns() - 1);
    table.with(Modify::new(selector).with(last_border));
    println!("{}", table);
}

fn looks_like_ticket(tag: String) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[A-Z0-9]*-[0-9]*$").unwrap();
    }
    RE.is_match(&tag)
}

fn print_with_dates_add_records(
    builder: &mut Builder,
    dates: &HashMap<NaiveDate, Vec<Vec<String>>>,
    dates_duration: &HashMap<NaiveDate, Duration>
) {
    let mut sorted_dates = dates.keys().collect::<Vec<&NaiveDate>>();
    sorted_dates.sort();
    for key in sorted_dates.iter()  {
        let mut first = vec![String::from(""); 6];
        first[0] = key.to_string();
        builder.add_record(first);

        for x in dates.get(key).unwrap().iter() {
            let mut row = vec![String::from("")];
            row.extend_from_slice(&x);
            builder.add_record(row);
        }

        let mut last = vec![String::from(""); 6];
        last[5] = duration_to_string(dates_duration.get(key).unwrap());
        builder.add_record(last);
    }
}

pub fn print_with_dates(results: Vec<DiaryDoc>) {
    let mut builder = Builder::default();
    builder.set_columns(
        vec!("Date", "Ticket", "Start", "End", "Duration", "Total")
    );

    let data = PrintWithDatesData::new(results);
    print_with_dates_add_records(&mut builder, &data.dates, &data.durations);

    let mut last = vec![String::from(""); 6];
    last[5] = duration_to_string(&data.total);
    builder.add_record(last);

    do_print(builder);
}

pub fn print(results: Vec<DiaryDoc>) {
    let mut builder = Builder::default();
    builder.set_columns(vec!("Ticket", "Log", "Duration"));
    let mut total = Duration::seconds(0);
    for x in results.iter() {
        let mut record = vec!(fname(x));
        let (rows, partial) = ranges(x);
        record.push(rows);
        record.push("".to_string());
        builder.add_record(record);
        builder.add_record(fmt_total(partial));
        total = total + partial;
    }
    builder.add_record(fmt_total(total));
    do_print(builder);
}

pub fn print_tags(results: Vec<DiaryDoc>) {
    let mut builder = Builder::default();
    builder.set_columns(vec!("Tag", "Duration"));
    let mut total = Duration::seconds(0);
    let mut tag_map: HashMap<String, Duration> = HashMap::new();
    for x in results.iter() {
        let (_, partial) = ranges(x);
        for tag in x.metadata.tags.iter() {
            let key = tag.to_string();
            let default = Duration::seconds(0);
            let current = tag_map.get(&key).unwrap_or(&default);
            tag_map.insert(key, *current + partial);
        }
        total = total + partial;
    }
    tag_map
        .iter()
        .filter(|x| !looks_like_ticket(String::from(x.0)))
        .map(|x| vec!(String::from(x.0), duration_to_string(x.1)))
        .for_each(|x| {
            builder.add_record(x);
        });
    builder.add_record(fmt_total(total));
    do_print(builder);
}
