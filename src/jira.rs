use chrono::{NaiveDateTime, Duration, offset::Local};
use reqwest::blocking::Client;
use std::{error::Error, fmt, ops::Range};
use serde::{Serialize, Deserialize};

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
    pub creator: JiraAuthor
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

#[derive(Debug)]
pub struct JiraError(String);

impl fmt::Display for JiraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JiraError: {}", self.0)
    }
}

impl Error for JiraError {}

pub fn fetch(key: &str) -> Result<JiraTicket, Box<dyn Error>> {
    let conf: Config = envy::prefixed("JIRA_").from_env().unwrap();
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

pub fn sync_worklog(
    key: String, 
    worklog: Range<NaiveDateTime>
) -> Result<(), Box<dyn Error>> {
    let duration: Duration = worklog.end - worklog.start;
    let conf: Config = envy::prefixed("JIRA_").from_env().unwrap();
    let vars = [
        ("notifyUsers", "false"),
        ("adjustEstimate", "auto"),
        ("overrideEditableFlag", "false")
    ];
    println!("original: {:?}", worklog.start);
    let started = worklog
        .start
        .and_local_timezone(Local::now().timezone())
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S%.3f%z")
        .to_string();
    let body = WorklogBody {
        comment: String::from(""),
        started,
        time_spent_seconds: duration.num_seconds(),
    };
    println!("{:?}", body);
    //let response = Client::new()
        //.post(format!("{}/rest/api/2/issue/{}/worklog", conf.host, key))
        //.header("Accept", "application/json")
        //.basic_auth(conf.user, Some(conf.pass))
        //.json(&body)
        //.query(&vars)
        //.send()?;
    //println!("{:?}", response);
    Ok(())
}
