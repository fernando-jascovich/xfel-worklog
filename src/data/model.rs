use chrono::Local;
use std::io::{Error, ErrorKind};
use chrono::NaiveDateTime;
use std::ops::Range;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl DiaryDoc {
    fn is_worklog_entry_complete(&self, entry: &Vec<&str>) -> bool {
        entry.len() > 1 && !entry[1].is_empty()
    }

    pub fn is_active(&self) -> bool {
        for x in &self.metadata.worklog {
            if !self.is_worklog_entry_complete(&self.worklog_entry(&x)) {
                return true;
            }
        }
        false
    }

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
            .filter_map(|x| self.worklog_to_date_range(x).ok())
            .collect()
    }

    fn worklog_entry<'a >(&self, worklog: &'a str) -> Vec<&'a str> {
        worklog.split(",").map(|x| x.trim()).collect()
    }

    pub fn worklog_to_date_range(
        &self, worklog: &str
    ) -> Result<Range<NaiveDateTime>, Box<dyn std::error::Error>> {
        let entry = self.worklog_entry(worklog);
        if !self.is_worklog_entry_complete(&entry) {
            return Err(
                Box::new(
                    Error::new(
                        ErrorKind::InvalidInput, "Invalid input string"
                    )
                )
            );
        }
        let fmt = "%Y-%m-%dT%H:%M:%S";
        let st = NaiveDateTime::parse_from_str(entry[0], fmt)?;
        let end = NaiveDateTime::parse_from_str(entry[1], fmt)?;
        Ok(Range { start: st, end })
    }

    pub fn start(&mut self) {
        let new_entry = format!(
            "{},", 
            Local::now().format("%Y-%m-%dT%H:%M:%S")
            );
        self.metadata.worklog.push(new_entry);
    }

    pub fn stop(&mut self) {
        let mut last_entry = self.metadata.worklog.pop().unwrap();
        last_entry = format!(
            "{}{}",
            last_entry,
            &mut Local::now().format("%Y-%m-%dT%H:%M:%S")
            );
        self.metadata.worklog.push(last_entry);
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
