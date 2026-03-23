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
    /// User agent index (0-4), defaults to 0 if not specified
    #[arg(short, long)]
    user_agent: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    info!("Starting search for query: {}", args.query);

    let html = request::run(&args.query, args.user_agent).await?;
    let results = parser::run(&html)?;

    if results.is_empty() {
        info!("No results found for query: {}", args.query);
    } else {
        info!("Found {} results", results.len());
    }

    for r in results {
        println!("Title: {}\nLink: {}", r.title, r.link);
        if let Some(content) = r.content {
            println!("Content: {}", content);
        }
        println!();
    }

    Ok(())
}
