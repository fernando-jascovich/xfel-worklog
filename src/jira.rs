use chrono::{DateTime, NaiveDateTime, Duration, offset::Local};
use reqwest::blocking::Client;
use std::{error::Error, fmt};
use serde::{Serialize, Deserialize};
use super::data::model::DiaryDoc;
use log::warn;

const JIRA_DATE_FMT: &str = "%Y-%m-%dT%H:%M:%S%.3f%z"; 

#[derive(Deserialize, Debug)]
struct Config {
    host: String,
    user: String,
    pass: String
}

#[derive(Serialize, Deserialize, Debug)]
struct WorklogBody {
    comment: String,
    started: String, // ISO Timestamp: 2021-01-17T12:34:00.000+0000

    #[serde(rename = "timeSpentSeconds")]
    time_spent_seconds: i64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraWorklog {
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: i64,

    pub started: String,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
struct JiraWorklogGetResponse {
    worklogs: Vec<JiraWorklog>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraTicket {
    #[serde(rename = "self")]
    pub url: String,

    pub key: String,
    pub fields: JiraTicketFields
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraTicketFields {
    pub description: Option<String>,
    pub summary: String,
    pub comment: JiraCommentHolder,
    pub creator: JiraAuthor,
    pub timetracking: JiraTimetracking
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraCommentHolder {
    pub comments: Vec<JiraComment>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraComment {
    pub author: JiraAuthor,
    pub body: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraAuthor {
    #[serde(rename = "displayName")]
    pub display_name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JiraTimetracking {
    #[serde(rename = "originalEstimate")]
    pub original_estimate: Option<String>
}

#[derive(Debug)]
pub struct JiraError(String);

impl fmt::Display for JiraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JiraError: {}", self.0)
    }
}

impl Error for JiraError {}

fn conf_from_env() -> Config {
    envy::prefixed("JIRA_").from_env().unwrap()
}

pub fn fetch(key: &str) -> Result<JiraTicket, Box<dyn Error>> {
    let conf = conf_from_env();
    let response = Client::new()
        .get(format!("{}/rest/api/2/issue/{}", conf.host, key))
        .header("Accept", "application/json")
        .basic_auth(conf.user, Some(conf.pass))
        .send()?;
    if !response.status().is_success() {
        return Err(Box::new(JiraError(response.text()?)));
    }
    Ok(response.json().unwrap())
}

fn worklog_uri(key: &str) -> String {
    format!("/rest/api/2/issue/{}/worklog", key)
}

pub fn fetch_worklogs(key: &str) -> Result<Vec<JiraWorklog>, Box<dyn Error>> {
    let conf: Config = conf_from_env();
    let response = Client::new()
        .get(format!("{}{}", conf.host, worklog_uri(key)))
        .basic_auth(conf.user, Some(conf.pass))
        .send()?;
    if !response.status().is_success() {
        return Err(Box::new(JiraError(response.text()?)));
    }
    Ok(response.json::<JiraWorklogGetResponse>().unwrap().worklogs)
}

fn ts_to_string(ts: NaiveDateTime) -> String {
    ts.and_local_timezone(Local::now().timezone())
        .unwrap()
        .format(JIRA_DATE_FMT)
        .to_string()
}

fn date_string_to_local_date_string(ts: &str) -> String {
    DateTime::parse_from_str(&ts, JIRA_DATE_FMT)
        .unwrap()
        .with_timezone(&Local::now().timezone())
        .format(JIRA_DATE_FMT)
        .to_string()
}

fn sync_worklog(
    key: &str, 
    started: &str,
    duration: &Duration
) -> Result<(), Box<dyn Error>> {
    let conf = conf_from_env();
    let vars = [
        ("notifyUsers", "false"),
        ("adjustEstimate", "auto"),
        ("overrideEditableFlag", "false")
    ];
    let body = WorklogBody {
        comment: String::from(""),
        started: started.to_string(),
        time_spent_seconds: duration.num_seconds(),
    };
    let response = Client::new()
        .post(format!("{}{}", conf.host, worklog_uri(key)))
        .header("Accept", "application/json")
        .basic_auth(conf.user, Some(conf.pass))
        .json(&body)
        .query(&vars)
        .send()?;
    if !response.status().is_success() {
        return Err(Box::new(JiraError(response.text()?)));
    }
    Ok(())
}

fn jira_key(doc: &DiaryDoc) -> String {
    let fname = doc.path.rsplit_once("/").unwrap().1;
    let without_extension = fname.rsplit_once(".").unwrap().0;
    without_extension.to_string()
}

pub fn sync_worklogs(doc: DiaryDoc) -> Result<(), Box<dyn Error>> {
    let key = jira_key(&doc);
    let current = fetch_worklogs(&key);
    if let Err(e) = current {
        return Err(e);
    }
    let existing: Vec<String> = current
        .unwrap()
        .iter()
        .map(|x| date_string_to_local_date_string(&x.started))
        .collect();
    for range in doc.worklog_range() {
        let started = ts_to_string(range.start);
        if existing.contains(&started) {
            warn!("Skipping existing entry: {}", started);
            continue
        }
        sync_worklog(&key, &started, &(range.end - range.start))?;
    }
    Ok(())
}
