use super::jira;
use super::data;
use log::{info, error};

pub fn run(key: &str, path: &Option<String>) {
    let result = jira::fetch(key);
    match result {
        Ok(ticket) => {
            let p = if let Some(path_str) = path {
                Some(path_str.as_str())
            } else {
                None
            }; 
            data::create_entry(ticket, p);
            let query_results = data::query::by_path(key);
            let doc = query_results.first().unwrap();
            info!("Created {}", doc.path);
        }
        Err(e) => error!("{}", e), 
    }
}
