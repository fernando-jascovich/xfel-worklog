use chrono::NaiveDate;
use super::{data, table, QueryOutput, stdin_path, print_paths};

pub fn run(
    tags: &Option<Vec<String>>,
    path: &Option<String>,
    start_date: &Option<NaiveDate>,
    end_date: &Option<NaiveDate>,
    output: &QueryOutput
) {
    let received_path = if let Some(p) = path {
        String::from(p)
    } else if let Some(p) = stdin_path() {
        String::from(p)
    } else {
        String::from("")
    };
    let results = if let Some(t) = tags {
        if let Some(st) = start_date {
            data::query::by_tags_and_date(t.clone(), &st, &end_date)
        } else {
            data::query::by_tags(t.clone())
        }
    } else if !received_path.is_empty() {
        if let Some(st) = start_date {
            data::query::by_path_and_date(&received_path, &st, &end_date)
        } else {
            data::query::by_path(&received_path)
        }
    } else if let Some(st) = start_date {
        data::query::by_date(&st, &end_date)
    } else {
        data::query::all()
    };
    match output {
        QueryOutput::Table => table::print(results),
        QueryOutput::Paths => print_paths(results)
    };
}

