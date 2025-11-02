mod cli;
mod models;
mod storage;
mod schedule;
mod daemon;
mod executor;
mod git;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
