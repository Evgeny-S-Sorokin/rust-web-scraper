use reqwest::header::{HeaderMap, HeaderValue};
use std::fs;

use log::{info, warn};

use crate::Result;
use crate::error::ScraperError;
use crate::useragent;

fn is_blocked(html: &str) -> bool {
    html.contains("/httpservice/retry/enablejs")
        || html.contains("challenge_version")
        || html.contains("SG_SS")
        || html.contains("anomaly.js")
        || html.contains("bots use DuckDuckGo")
}

pub async fn run(query: &str, user_agent_index: Option<usize>) -> Result<String> {
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

    let mut headers = HeaderMap::new();
    headers.insert(
        "User-Agent",
        HeaderValue::from_static(useragent::get_user_agent(user_agent_index.unwrap_or(0))),
    );
    headers.insert(
        "Accept",
        HeaderValue::from_static("text/html,application/xhtml+xml"),
    );
    headers.insert(
        "Accept-Language",
        HeaderValue::from_static("en-US,en;q=0.9"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client
        .get("https://duckduckgo.com/html/")
        .query(&[("q", query), ("hl", "en"), ("num", "10")])
        .send()
        .await?;

    if !response.status().is_success() {
        warn!("HTTP error: {}", response.status());
        return Err(ScraperError::Parse(format!(
            "Failed to fetch search results: HTTP {}",
            response.status()
        )));
    }

    let text = response.text().await?;
    if is_blocked(&text) {
        warn!("Google is blocking requests - received CAPTCHA page");
        let _ = fs::write("request_debug_html.html", text);
        info!("Saved HTML to request_debug_html.html for inspection");
        return Err(ScraperError::Parse(
            "Google is blocking requests - received CAPTCHA page".to_string(),
        ));
    }

    info!("Successfully fetched search results");
    Ok(text)
}
