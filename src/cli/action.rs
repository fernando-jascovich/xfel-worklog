use super::{data, ActionKind, stop_active_docs, stdin_path};
use super::data::model::DiaryDoc;
use super::jira;
use log::{info, error};

fn do_action(kind: &ActionKind, mut doc: DiaryDoc) {
    match kind {
        ActionKind::Start => {
            if doc.is_active() {
                error!("Requested doc is already active");
                return;
            }
            stop_active_docs();
            doc.start();
            data::update_entry(doc);
        }
        ActionKind::Stop => {
            if !doc.is_active() {
                error!("Requested doc is not active");
                return;
            }
            doc.stop();
            data::update_entry(doc);
        }
        ActionKind::SyncWorklog => {
            info!("Syncing worklogs for {}...", doc.path);
            if let Err(e) = jira::sync_worklogs(doc) {
                error!("{}", e);
            };
            info!("Finished");
        }
    };
}

pub fn run(path: &Option<String>, kind: &ActionKind) {
    let results = if let Some(p) = path  {
        data::query::by_path(p)
    } else {
        data::query::by_path(&stdin_path().unwrap())
    };
    let matched_docs = results.len();

    if matched_docs < 1 {
        error!("Path doesn't match any document");
        return;
    }
    info!("Query matched {} docs", matched_docs);
    for doc in results {
        do_action(kind, doc);
    }
}
