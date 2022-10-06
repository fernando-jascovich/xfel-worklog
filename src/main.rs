mod model;
mod query;
mod data;

use chrono::Duration;
use clap::Parser;
use model::{Cli, Commands, DiaryDoc};
use log::info;
use tabled::{builder::Builder};

fn browse() {
    let paths: Vec<String> = query::all()
        .iter()
        .map(|x| String::from(&x.path))
        .collect();
    info!("{:?}", paths)
}

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
    let cli = Cli::parse();

    match &cli.command {
        Commands::Query { tags, path, start_date, end_date } => {
            info!(
                "tags: {:?}, path: {:?}, start_date: {:?}, end_date: {:?}",
                tags,
                path,
                start_date,
                end_date
            );
            let results = if let Some(t) = tags {
                if let Some(st) = start_date {
                    query::by_tags_and_date(t.clone(), st, end_date)
                } else {
                    query::by_tags(t.clone())
                }
            } else if let Some(p) = path {
                query::by_path(p)
            } else if let Some(st) = start_date {
                query::by_date(st, end_date)
            } else {
                query::all()
            };
            print_results(results);
        }
        Commands::Action { path, action } => {
            info!("action: {:?}, {:?}", path, action);
        }
        Commands::Browse => browse()
    }
}
