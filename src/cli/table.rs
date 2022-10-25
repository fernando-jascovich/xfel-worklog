use std::{collections::HashMap, ops::Range};
use lazy_static::lazy_static;
use regex::Regex;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use tabled::locator::ByColumnName;
use super::data::model::DiaryDoc;
use tabled::object::Rows;
use tabled::builder::Builder;
use tabled::{Style, Modify, Border, Panel, Alignment};

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

fn do_print(builder: Builder, total: &Duration) {
    let mut table = builder.build();
    table.with(Style::psql());

    let rows = table.count_rows();
    let cols = table.count_columns();
    let records = table.get_records().clone();

    let mut cursor = 0;
    while cursor < rows {
        let coord = (cursor, cols - 1);
        let cell = &records[coord];
        if !cell.is_empty() {
            let border = Border::empty().bottom('-');
            let selector = Rows::single(coord.0);
            table.with(Modify::new(selector).with(border));
        }
        cursor += 1;
    }
    table.with(Modify::new(Rows::last()).with(Border::empty().bottom('=')));

    for col_name in ["Duration", "Start", "End", "Total"] {
        table.with(
            Modify::new(ByColumnName::new(col_name)).with(Alignment::right())
        );
    }
    table.with(
        Modify::new(Rows::first())
            .with(Alignment::center())
            .with(Border::empty().bottom('=').top('='))
    );
    let footer_msg = format!("Total: {}", duration_to_string(&total));
    table.with(Panel::footer(footer_msg));
    table.with(Modify::new(Rows::last()).with(Alignment::right()));

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

    do_print(builder, &data.total);
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
    do_print(builder, &total);
}
