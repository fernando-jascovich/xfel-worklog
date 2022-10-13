mod table;

use std::io;
use atty::Stream;
use clap::{Parser, Subcommand};
use chrono::NaiveDate;
use chrono::offset::Local;
use super::data;
use super::jira;
use log::{info, error};

fn default_start_date() -> &'static str {
    let today = Local::today().format("%Y-%m-%d");
    Box::leak(
        format!("{}", today).into_boxed_str()
    )
}

fn stdin_path() -> Option<String> {
    if atty::is(Stream::Stdin) {
        return None;
    }
    io::stdin().lines().nth(0).unwrap().ok()
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
        /// It will default to received stdin if any. 
        /// When stdin contains more than a line, it will consider
        /// only the first line of it.
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
        /// Path on which operate. It should point to a single element.
        /// It will default to received stdin if any. 
        /// When stdin contains more than a line, it will consider
        /// only the first line of it.
        #[arg(short, long)]
        path: Option<String>,

        #[command(subcommand)]
        kind: ActionKind
    },

    /// Similar to query but this will return a list of matched paths
    Browse {
        /// Return only active paths. This is, files with an unterminated
        /// worklog.
        #[arg(short, long, default_value_t = false)]
        active: bool
    },

    /// Fetch element from Jira.
    Fetch {
        /// Issue key
        key: String,

        /// Optional path into DATA_ROOT
        #[arg(short, long)]
        path: Option<String>
    },

    /// Sync element's worklog with jira
    SyncWorklog {}
}

#[derive(Subcommand, Debug)]
pub enum ActionKind {
    Start,
    Stop
}

fn query(
    tags: &Option<Vec<String>>,
    path: &Option<String>,
    start_date: &Option<NaiveDate>,
    end_date: &Option<NaiveDate>
) {
    let results = if let Some(t) = tags {
        if let Some(st) = start_date {
            data::query::by_tags_and_date(t.clone(), &st, &end_date)
        } else {
            data::query::by_tags(t.clone())
        }
    } else if let Some(p) = path {
        data::query::by_path(&p)
    } else if let Some(p) = stdin_path() {
        data::query::by_path(&p)
    } else if let Some(st) = start_date {
        data::query::by_date(&st, &end_date)
    } else {
        data::query::all()
    };
    table::print(results);
}

fn action(path: &Option<String>, kind: &ActionKind) {
    let results = if let Some(p) = path  {
        data::query::by_path(p)
    } else {
        data::query::by_path(&stdin_path().unwrap())
    };
    let mut doc = results.first().unwrap().clone();
    match kind {
        ActionKind::Start => {
            if doc.is_active() {
                error!("Requested doc is already active");
                return;
            }
            doc.start();
            data::update_entry(doc);
        }
        ActionKind::Stop => {
            if !doc.is_active() {
                error!("Requested doc is not active");
                return;
            }
            doc.stop();
            data::update_entry(doc);
        }
    };
}

fn browse(active: &bool) {
    let mut docs: Vec<data::model::DiaryDoc> = data::query::all();
    if *active {
        docs.retain(|x| x.is_active());
    }
    let paths: Vec<String> = docs
        .iter()
        .map(|x| String::from(&x.path))
        .collect();
    for x in paths {
        println!("{}", x);
    }
}

fn fetch(key: &str, path: &Option<String>) {
    let result = jira::fetch(key);
    match result {
        Ok(ticket) => {
            let p = if let Some(path_str) = path {
                Some(path_str.as_str())
            } else {
                None
            }; 
            data::create_entry(ticket, p);
            let query_results = data::query::by_path(key);
            let doc = query_results.first().unwrap();
            info!("Created {}", doc.path);
        }
        Err(e) => error!("{}", e), 
    }
}

pub fn main() {
    let cli = Args::parse();
    match &cli.command {
        Commands::Query { tags, path, start_date, end_date } => {
            query(tags, path, start_date, end_date);
        }
        Commands::Action { path, kind } => action(path, kind),
        Commands::Browse { active } => browse(active),
        Commands::Fetch { key, path } => fetch(key, path),
        Commands::SyncWorklog { } => todo!()
    }
}
