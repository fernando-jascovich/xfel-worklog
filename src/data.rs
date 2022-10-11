pub mod model;
pub mod query;

use log::{warn, info};
use model::DiaryDoc;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use super::jira::JiraTicket;
use walkdir::WalkDir;
use yaml_front_matter::YamlFrontMatter;

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

fn replace_frontmatter(content: &str, new_fm: &str) -> String {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let fm_end = end + 6; // 3 for opening and 3 for closure
            return format!("---\n{}\n---\n{}", new_fm, &content[fm_end..]);
        } else {
            warn!("Unterminatted frontmatter detected");
        }
    } else {
        info!("No frontmatter detected");
    }
    return format!("---\n{}\n---\n{}", new_fm, content)
}

pub fn update_entry(doc: DiaryDoc) {
    info!("Updating: {}", doc.path);
    let mut file = File::open(doc.path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let new_contents = replace_frontmatter(&contents, "bablavbabsbbbbbbbb");
    info!("New: {}", new_contents);
}

pub fn create_entry(ticket: JiraTicket, base_path: Option<&str>) {
    let key_parts: Vec<&str> = ticket.key.split("-").collect();
    let mut tags = format!("'{}', '{}'", key_parts[0], ticket.key);
    let mut dir = key_parts[0].to_string();

    if let Some(base) = base_path {
        tags += format!(", '{}'", base).as_str();
        dir = format!("{}/{}", base, key_parts[0]);
    } 

    let mut file = File::create(
        format!("{}/{}/{}.md", conf().root, dir, ticket.key)
    ).unwrap();
    let file_data = format!(
        r#"---
author: '{}'
date: ''
tags: [{}]
estimate: 
worklog:
---
# {}

{}
"#,
        ticket.fields.creator.display_name, 
        tags,
        ticket.fields.summary, 
        ticket.fields.description.unwrap_or("".to_string())
    );
    file.write_all(file_data.as_bytes()).unwrap();
}

