use clap::Parser;
use log::info;

mod error;
mod parser;
mod request;
mod useragent;

type Result<T> = std::result::Result<T, error::ScraperError>;

#[derive(Parser)]
struct Args {
    query: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    info!("Starting search for query: {}", args.query);

    let html = request::run(&args.query).await?;
    let results = parser::run(&html)?;

    if results.is_empty() {
        info!("No results found for query: {}", args.query);
    } else {
        info!("Found {} results", results.len());
    }

    for r in results {
        println!("Title: {}\nLink: {}\n", r.title, r.link);
    }

    Ok(())
}
