use clap::{Parser, Subcommand};
use chrono::NaiveDate;
use chrono::offset::Utc;
use super::query;
use super::data;
use super::jira;
use super::model::DiaryDoc;

fn default_start_date() -> &'static str {
    let today = Utc::today().format("%Y-%m-%d");
    Box::leak(
        format!("{}", today).into_boxed_str()
    )
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query for elements
    Query {
        /// Filter by tags: 'my tag'
        #[arg(short, long)]
        tags: Option<Vec<String>>,

        /// Filter by path, it could by any part of the diary's path
        #[arg(short, long)]
        path: Option<String>,

        /// In ISO format: '2020-01-01'
        #[arg(default_value = default_start_date())]
        start_date: Option<NaiveDate>,

        /// In ISO format: '2020-01-01'
        end_date: Option<NaiveDate>
    },

    /// Perform an action on elements
    Action {
        #[arg(short, long)]
        path: String,

        #[command(subcommand)]
        action: Action
    },

    /// Similar to query but this will return a list of matched paths
    Browse,

    /// Fetch element from Jira.
    Fetch {
        /// Issue key
        key: String
    }
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Start,
    Stop
}

pub struct Output {
    pub diaries: Vec<DiaryDoc>
}

pub fn main() -> Output {
    let cli = Args::parse();
    match &cli.command {
        Commands::Query { tags, path, start_date, end_date } => {
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
            Output { diaries: results }
        }
        Commands::Action { path, action } => {
            todo!()
        }
        Commands::Browse => {
            let paths: Vec<String> = query::all()
                .iter()
                .map(|x| String::from(&x.path))
                .collect();
            todo!()
        }
        Commands::Fetch { key } => {
            let ticket = jira::fetch(key).unwrap();
            data::create_entry(ticket, Some("gfr"));
            Output { diaries: query::by_path(key) }
        }
    }
}
