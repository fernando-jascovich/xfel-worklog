use std::fs::File;
use std::io::prelude::*;
use walkdir::WalkDir;
use yaml_front_matter::YamlFrontMatter;
use std::fs;
use log::warn;
use serde::{Serialize, Deserialize};
use super::model::DiaryDoc;
use super::jira::JiraTicket;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    root: String
}

fn conf() -> Config {
    envy::prefixed("DATA_").from_env().unwrap()
}


pub fn load_diary() -> Vec<DiaryDoc> {
    let mut output: Vec<DiaryDoc> = Vec::new();
    let iter = WalkDir::new(conf().root) 
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir());
    for entry in iter {
        let path = String::from(entry.path().to_str().unwrap());
        let md = fs::read_to_string(&path).unwrap();
        match YamlFrontMatter::parse(&md) {
            Ok(doc) => {
                output.push(DiaryDoc {
                    metadata: doc.metadata,
                    path: path
                });
            }
            Err(e) => {
                warn!("Error with path {}", path);
                warn!("{}", e);
            }
        }
    }
    output
}

pub fn create_entry(ticket: JiraTicket, base_path: Option<&str>) {
    let key_parts: Vec<&str> = ticket.key.split("-").collect();
    let dir = if let Some(base) = base_path {
        format!("{}/{}", base, key_parts[0])
    } else {
        key_parts[0].to_string()
    };
    let mut file = File::create(
        format!("{}/{}/{}.md", conf().root, dir, ticket.key)
    ).unwrap();
    let file_data = format!(r#"---
author: '{}'
date: ''
tags: ['{}', '{}']
estimate: 
worklog:
---
# {}

{}
"#, 
        ticket.fields.creator.display_name, 
        key_parts[0], 
        ticket.key, 
        ticket.fields.summary, 
        ticket.fields.description
    );
    file.write_all(file_data.as_bytes());
}

