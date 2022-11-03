pub mod model;
pub mod query;

use std::fs::OpenOptions;
use log::{warn, info};
use model::{DiaryDoc,Metadata};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use super::jira::JiraTicket;
use walkdir::{WalkDir, DirEntry};
use yaml_front_matter::YamlFrontMatter;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    root: String,
    include_archive: Option<bool>
}

fn conf() -> Config {
    envy::prefixed("DIARY_").from_env().unwrap()
}


pub fn load_diary() -> Vec<DiaryDoc> {
    let mut output: Vec<DiaryDoc> = Vec::new();
    let mut iter: Vec<DirEntry> = WalkDir::new(conf().root) 
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| !e.path().to_str().unwrap().contains(".git"))
        .collect();

    let include_archive = conf().include_archive.unwrap_or(false);
    if !include_archive {
        iter = iter
            .into_iter()
            .filter(|e| !e.path().to_str().unwrap().contains("_archive"))
            .collect();
    }
    
    for entry in iter {
        let path = String::from(entry.path().to_str().unwrap());
        let md = fs::read_to_string(&path).unwrap();
        match YamlFrontMatter::parse(&md) {
            Ok(doc) => {
                output.push(DiaryDoc {
                    metadata: doc.metadata,
                    path
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
            return format!("---\n{}\n---{}", new_fm, &content[fm_end..]);
        } else {
            warn!("Unterminatted frontmatter detected");
        }
    } else {
        info!("No frontmatter detected");
    }
    return format!("---\n{}\n---{}", new_fm, content)
}

pub fn update_entry(doc: DiaryDoc) {
    info!("Updating: {}", doc.path);
    let mut file_r = File::open(&doc.path).unwrap();
    let mut contents = String::new();
    file_r.read_to_string(&mut contents).unwrap();
    let yaml = serde_yaml::to_string(&doc.metadata).unwrap();
    let new_contents = replace_frontmatter(&contents, &yaml);

    let mut file_w = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&doc.path)
        .unwrap();
    file_w.write_all(new_contents.as_bytes()).unwrap();
}

pub fn create_entry(ticket: JiraTicket, base_path: Option<&str>) {
    let key_parts: Vec<&str> = ticket.key.split("-").collect();
    let mut tags = vec!(key_parts[0].to_string(), ticket.key.to_string());
    let mut dir = key_parts[0].to_string();
    if let Some(base) = base_path {
        tags.push(base.to_string());
        dir = format!("{}/{}", base, key_parts[0]);
    } 
    let metadata = Metadata {
        author: Some(ticket.fields.creator.display_name),
        date: None,
        tags,
        estimate: ticket.fields.timetracking.original_estimate,
        worklog: vec!()
    };
    let path = format!("{}/{}/{}.md", conf().root, dir, ticket.key);
    info!("Writing entry into {}", path);
    let mut file = File::create(path).unwrap();
    let comments: String = ticket.fields.comment.comments
        .iter()
        .map(|x| format!("## {}\n{}", x.author.display_name, x.body))
        .collect::<Vec<String>>()
        .join("\n\n");
    let file_data = format!(
        "---\n{}---\n# {}\n\n{}\n\n# Comments\n\n{}",
        serde_yaml::to_string(&metadata).unwrap(),
        ticket.fields.summary, 
        ticket.fields.description.unwrap_or("".to_string()),
        comments
    );
    file.write_all(file_data.as_bytes()).unwrap();
}

