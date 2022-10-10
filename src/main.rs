mod data;
mod cli;
mod jira;

fn main() {
    simple_logger::init_with_env().unwrap();
    dotenv::dotenv().ok();
    cli::main();
}
