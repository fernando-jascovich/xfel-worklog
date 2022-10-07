mod model;
mod query;
mod data;
mod jira;
mod cli;

use chrono::Duration;
use model::DiaryDoc;
use tabled::{builder::Builder};

fn duration_to_string(duration: Duration) -> String {
    String::from(
        format!(
            "{}h {}m", 
            duration.num_hours(), 
            duration.num_minutes() - duration.num_hours() * 60
        )
    )
}

fn print_results(results: Vec<DiaryDoc>) {
    let mut builder = Builder::default();
    let mut total = Duration::seconds(0);
    for x in results.iter() {
        let fname = String::from(x.path.split("/").last().unwrap()); 
        let mut record = vec!(fname);
        let mut partial = Duration::seconds(0);
        let ranges: String = x.worklog_range()
            .iter()
            .map(|y| {
                let diff = y.end - y.start;
                let dur = duration_to_string(y.end - y.start);
                partial = partial + diff;
                format!("{} -> {}: {}", y.start, y.end, dur)
            })
            .collect::<Vec<String>>()
            .join("\n");
        record.push(ranges);
        builder.add_record(record);
        builder.add_record(vec!("".to_string(), "".to_string(), duration_to_string(partial)));
        total = total + partial;
    }
    builder.add_record(vec!("".to_string(), "".to_string(), duration_to_string(total)));
    let table = builder.build();
    println!("{}", table);
}

fn main() {
    simple_logger::init_with_env().unwrap();
    dotenv::dotenv().ok();
    let output: cli::Output = cli::main();
    print_results(output.diaries);
}
