use walkdir::WalkDir;
use yaml_front_matter::YamlFrontMatter;
use std::fs;
use log::warn;
use super::model::DiaryDoc;

const ROOT_DIR: &str = "/home/xfel/diary";

pub fn load_diary() -> Vec<DiaryDoc> {
    let mut output: Vec<DiaryDoc> = Vec::new();
    let iter = WalkDir::new(ROOT_DIR) 
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

