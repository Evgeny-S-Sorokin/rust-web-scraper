use std::time::Duration;

use log::{info, warn};

use crate::Result;
use crate::error::ScraperError;
use crate::useragent::random_user_agent;

pub async fn run(query: &str) -> Result<String> {
    info!("Fetching search results for: {}", query);

    if query.is_empty() {
        warn!("Query validation failed: empty query");
        return Err(ScraperError::Parse("Query cannot be empty".to_string()));
    }

    if query.len() > 2048 {
        warn!("Query validation failed: exceeds max length");
        return Err(ScraperError::Parse(
            "Query exceeds maximum length of 2048 characters".to_string(),
        ));
    }

    let url = format!(
        "https://www.google.com/search?q={}",
        urlencoding::encode(query)
    );
    info!("Request URL: {}", url);

    let client = reqwest::Client::builder()
        .user_agent(random_user_agent())
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(10)
        .build()?;

    let response = client.get(&url).send().await?;
    info!("Response status: {}", response.status());

    if !response.status().is_success() {
        warn!("HTTP error: {}", response.status());
        return Err(ScraperError::Parse(format!(
            "Failed to fetch search results: HTTP {}",
            response.status()
        )));
    }

    info!("Successfully fetched search results");
    response.text().await.map_err(ScraperError::from)
}
