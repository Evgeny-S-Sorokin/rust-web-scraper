use log::{info, warn};
use scraper::{Html, Selector};
use std::fs;

use crate::Result;
use crate::error::ScraperError;

pub struct SearchResult {
    pub title: String,
    pub link: String,
}

pub fn run(html: &str) -> Result<Vec<SearchResult>> {
    info!("Starting to parse HTML content");

    if html.is_empty() {
        warn!("Parse failed: HTML content is empty");
        return Err(ScraperError::Parse("HTML content is empty".to_string()));
    }

    info!("HTML length: {} bytes", html.len());

    let doc = Html::parse_document(html);
    let link_selector = Selector::parse("a.result__a").unwrap();

    if doc.select(&link_selector).count() == 0 {
        warn!("No links selected");
        let _ = fs::write("parse_debug_html.html", html);
        info!("Saved HTML to parse_debug_html.html for inspection");
        return Err(ScraperError::Parse("No links selected".to_string()));
    }

    let results = doc
        .select(&link_selector)
        .filter_map(|link_elem| {
            let title = link_elem.text().collect::<String>();
            let link = link_elem.value().attr("href")?.to_string();

            Some(SearchResult { title, link })
        })
        .collect::<Vec<_>>();

    info!("Successfully parsed {} search results", results.len());
    Ok(results)
}
