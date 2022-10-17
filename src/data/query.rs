use chrono::{NaiveDate, NaiveDateTime};
use super::model::DiaryDoc;
use super::load_diary;

fn sort_by_date(data: &mut Vec<DiaryDoc>) {
    data.sort_by_key(|x| {
        let range = x.worklog_range();
        if range.len() < 1 {
            0
        } else {
            range[0].start.timestamp()
        }
    });
}

pub fn all() -> Vec<DiaryDoc> {
    load_diary().clone()
}

pub fn active() -> Vec<DiaryDoc> {
    let mut data = load_diary();
    data.retain(|x| x.is_active());
    data
}

pub fn by_tags(tags: Vec<String>) -> Vec<DiaryDoc> {
    let mut data: Vec<DiaryDoc> = load_diary();
    data.retain(|x| {
        let mut item_tags = x.metadata.tags.clone();
        item_tags.retain(|y| tags.contains(y));
        item_tags.len() > 0
    });
    data
}

pub fn by_tags_and_date(
    tags: Vec<String>, 
    start_date: &NaiveDate, 
    end_date: &Option<NaiveDate>
) -> Vec<DiaryDoc> {
    let data: Vec<DiaryDoc> = by_tags(tags);
    filter_date(data, start_date, end_date)
}

fn filter_date(
    mut data: Vec<DiaryDoc>,
    start_date: &NaiveDate, 
    end_date: &Option<NaiveDate>
) -> Vec<DiaryDoc> {
    let st_ts = start_date.and_hms(0, 0, 0);
    let end_ts: Option<NaiveDateTime> = if let Some(end) = end_date {
        Some(end.and_hms(23, 59, 59))
    } else {
        None
    };
    data.retain(|x| {
        if !x.has_work_after(&st_ts) {
            return false;
        }
        if let Some(end) = end_ts {
            return x.has_work_before(&end);
        }
        true
    });
    for doc in data.iter_mut() {
        let mut worklog: Vec<String> = doc.metadata.worklog.to_vec();
        worklog.retain_mut(|x| {
            if let Some(range) = doc.worklog_to_date_range(x).ok() {
                if let Some(end) = end_ts {
                    return range.start > st_ts && range.end < end;
                }
                return range.start > st_ts;
            }
            false
        });
        doc.metadata.worklog = worklog;
    }
    data.retain(|x| x.metadata.worklog.len() > 0);
    sort_by_date(&mut data);
    data
}

pub fn by_date(
    start_date: &NaiveDate, 
    end_date: &Option<NaiveDate>
) -> Vec<DiaryDoc> {
    let data: Vec<DiaryDoc> = load_diary();
    filter_date(data, start_date, end_date)
}

pub fn by_path(path: &str) -> Vec<DiaryDoc> {
    let mut data: Vec<DiaryDoc> = load_diary();
    data.retain(|x| x.path.contains(path));
    data
}

pub fn by_path_and_date(
    path: &str, 
    start_date: &NaiveDate, 
    end_date: &Option<NaiveDate>
) -> Vec<DiaryDoc> {
    let data: Vec<DiaryDoc> = by_path(path);
    filter_date(data, start_date, end_date)
}
