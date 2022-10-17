use super::{data, print_paths};

pub fn run(active: &bool) {
    let docs = if *active {
        data::query::active()
    } else {
        data::query::all()
    };
    print_paths(docs);
}

