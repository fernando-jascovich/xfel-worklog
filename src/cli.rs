mod table;
mod query;
mod action;
mod browse;
mod fetch;
mod date_parse;

use std::io;
use atty::Stream;
use clap::{Parser, Subcommand, ValueEnum};
use chrono::NaiveDate;
use chrono::offset::Local;
use super::data;
use super::jira;
use log::info;

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

        /// Output only paths, similar to browse command
        #[arg(short, long, value_enum, default_value_t = QueryOutput::Table)]
        output: QueryOutput,

        /// In ISO format: '2020-01-01'
        #[arg(
            default_value = default_start_date(), 
            value_parser = date_parse::input
        )]
        start_date: Option<NaiveDate>,

        /// In ISO format: '2020-01-01'
        end_date: Option<String>
    },

    /// Perform an action on elements
    Action {
        /// Path on which operate. It should point to a single element.
        /// It will default to received stdin if any. 
        /// When stdin contains more than a line, it will consider
        /// only the first line of it.
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
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum QueryOutput {
    /// It will output a table with files and time spent.
    Table,

    /// It will output a list of matched files.
    Paths,

    /// It will output a table only with tags and time spent.
    /// Note that tags that looks like Jira tickets (EXAMPLE-123)
    /// will be filtered out from output (not from time sum)
    Tags
}

#[derive(Subcommand, Debug)]
pub enum ActionKind {
    Start,
    Stop,
    SyncWorklog
}

fn stop_active_docs() {
    for mut active_doc in data::query::active() {
        info!("Stopping active doc: {}", active_doc.path);
        active_doc.stop();
        data::update_entry(active_doc);
    }
}

fn print_paths(docs: Vec<data::model::DiaryDoc>) {
    let paths: Vec<String> = docs
        .iter()
        .map(|x| String::from(&x.path))
        .collect();
    for x in paths {
        println!("{}", x);
    }
}

pub fn main() {
    let cli = Args::parse();
    match &cli.command {
        Commands::Query { tags, path, start_date, end_date, output } => {
            let end_date_parsed: Option<NaiveDate> = if let Some(x) = end_date {
                date_parse::input(x).ok()
            } else {
                None
            };
            query::run(tags, path, start_date, &end_date_parsed, output);
        }
        Commands::Action { path, kind } => action::run(path, kind),
        Commands::Browse { active } => browse::run(active),
        Commands::Fetch { key, path } => fetch::run(key, path)
    }
}
