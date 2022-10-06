use reqwest::blocking::Client;
use std::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
struct Config {
    host: String,
    user: String,
    pass: String
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
    pub description: String,
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

pub fn fetch(key: &str) -> Result<JiraTicket, Box<dyn Error>> {
    let conf: Config = envy::prefixed("JIRA_").from_env().unwrap();
    let response = Client::new()
        .get(format!("{}/rest/api/2/issue/{}", conf.host, key))
        .header("Accept", "application/json")
        .basic_auth(conf.user, Some(conf.pass))
        .send()?;
    Ok(response.json().unwrap())
}
