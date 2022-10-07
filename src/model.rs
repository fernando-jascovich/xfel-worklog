use std::io::{Error, ErrorKind};
use chrono::NaiveDateTime;
use std::ops::Range;
use std::fmt;
use serde::{Deserialize};

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
            .filter_map(|x| self.worklog_to_date_range(x).ok())
            .collect()
    }

    pub fn worklog_to_date_range(
        &self, worklog: &str
    ) -> Result<Range<NaiveDateTime>, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = worklog.split(",").map(|x| x.trim()).collect();
        if parts.len() < 2 {
            return Err(
                Box::new(
                    Error::new(
                        ErrorKind::InvalidInput, "Invalid input string"
                    )
                )
            );
        }
        let fmt = "%Y-%m-%dT%H:%M:%S";
        let st = NaiveDateTime::parse_from_str(parts[0], fmt)?;
        let end = NaiveDateTime::parse_from_str(parts[1], fmt)?;
        Ok(Range { start: st, end: end })
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
