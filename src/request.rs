use reqwest::Url;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderMap, HeaderValue};
use std::fs;

use log::{info, warn};

use crate::Result;
use crate::error::ScraperError;
use crate::useragent;

fn is_blocked(html: &str) -> bool {
    html.contains("Your privacy choices") || html.contains("consent-page")
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
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(
        "Accept-Language",
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        "Accept-Encoding",
        HeaderValue::from_static("gzip, deflate, br"),
    );

    let jar = Jar::default();
    let url = Url::parse("https://search.yahoo.com").unwrap();
    let sb = "v=1&vm=p&fl=1&vl=lang_en&pn=10&rw=new&userset=1";
    jar.add_cookie_str(&format!("sB={}", sb), &url);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .cookie_provider(std::sync::Arc::new(jar))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client
        .get("https://search.yahoo.com/search")
        .query(&[("p", query), ("iscqry", "")])
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
        warn!("Engine is blocking requests - received blocker page");
        let _ = fs::write("request_debug_html.html", text);
        info!("Saved HTML to request_debug_html.html for inspection");
        return Err(ScraperError::Parse(
            "Engine is blocking requests - received blocker page".to_string(),
        ));
    }

    info!("Successfully fetched search results");
    Ok(text)
}
