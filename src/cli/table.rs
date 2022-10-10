use chrono::Duration;
use super::data::model::DiaryDoc;
use tabled::{builder::Builder};

fn duration_to_string(duration: Duration) -> String {
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
            let dur = duration_to_string(y.end - y.start);
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
        duration_to_string(duration)
    )
}

pub fn print(results: Vec<DiaryDoc>) {
    let mut builder = Builder::default();
    let mut total = Duration::seconds(0);
    for x in results.iter() {
        let mut record = vec!(fname(x));
        let (rows, partial) = ranges(x);
        record.push(rows);
        builder.add_record(record);
        builder.add_record(fmt_total(partial));
        total = total + partial;
    }
    builder.add_record(fmt_total(total));
    let table = builder.build();
    println!("{}", table);
}

