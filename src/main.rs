mod model;
mod query;
mod data;

use clap::Parser;
use model::{Cli, Commands};
use log::info;

fn browse() {
    let paths: Vec<String> = query::all()
        .iter()
        .map(|x| String::from(&x.path))
        .collect();
    info!("{:?}", paths)
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
            info!("results: {:?}", results);
        }
        Commands::Action { path, action } => {
            info!("action: {:?}, {:?}", path, action);
        }
        Commands::Browse => browse()
    }
}
