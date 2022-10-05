use chrono::{NaiveDateTime, NaiveDate};
use std::ops::Range;
use std::fmt;
use serde::{Deserialize};
use clap::{Parser, Subcommand};

#[derive(Deserialize, Debug, Clone)]
pub struct Metadata {
    pub author: Option<String>,
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub estimate: Option<String>,
    pub worklog: Vec<String>
}

#[derive(Clone)]
pub struct DiaryDoc {
    pub metadata: Metadata,
    pub path: String
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Query {
        #[arg(short, long)]
        tags: Option<Vec<String>>,

        #[arg(short, long)]
        path: Option<String>,

        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>
    },
    Action {
        #[arg(short, long)]
        path: String,

        #[command(subcommand)]
        action: Action
    },
    Browse
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Start,
    Stop,
    Fetch
}

fn worklog_to_date_range(worklog: &str) -> Range<NaiveDateTime> {
    let parts: Vec<&str> = worklog.split(",").map(|x| x.trim()).collect();
    let fmt = "%Y-%m-%dT%H:%M:%S";
    Range {
        start: NaiveDateTime::parse_from_str(parts[0], fmt).unwrap(),
        end: NaiveDateTime::parse_from_str(parts[1], fmt).unwrap()
    }
}

impl DiaryDoc {
    pub fn has_work_after(&self, datetime: &NaiveDateTime) -> bool {
        for range in self.worklog_range() {
            if &range.end > datetime {
                return true;
            }
        }
        false
    }

    pub fn has_work_before(&self, datetime: &NaiveDateTime) -> bool {
        for range in self.worklog_range() {
            if &range.start < datetime {
                return true;
            }
        }
        false
    }

    pub fn worklog_range(&self) -> Vec<Range<NaiveDateTime>> {
        self.metadata.worklog
            .iter()
            .filter(|x| x.contains(","))
            .map(|x| worklog_to_date_range(x))
            .collect()
    }
}

impl fmt::Display for DiaryDoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl fmt::Debug for DiaryDoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}
