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

    if (html.contains("noscript") || html.contains("<script")) && html.len() < 10000 {
        warn!("Page appears to be JS-heavy - Google likely requires JavaScript rendering");
        let _ = fs::write("debug_html.html", html);
        info!("Saved HTML to debug_html.html for inspection");
        return Err(ScraperError::Parse(
            "Page requires JS rendering".to_string(),
        ));
    }

    let doc = Html::parse_document(html);

    let selectors = [
        "div.g",
        "div[data-sokoban-container]",
        "div[data-result-container]",
    ];

    let results_selector = selectors
        .iter()
        .find_map(|s| Selector::parse(s).ok())
        .ok_or_else(|| ScraperError::Parse("Failed to parse any results selector".to_string()))?;
    let found_count = doc.select(&results_selector).count();
    info!("Found {} result containers", found_count);

    if found_count == 0 {
        info!("No results found with current selectors. Inspecting page structure...");

        if let Ok(sel) = Selector::parse("body")
            && doc.select(&sel).count() > 0
        {
            info!("Page has <body> element");
        }

        if let Ok(sel) = Selector::parse("[role=\"main\"]") {
            let count = doc.select(&sel).count();
            info!("Found {} [role=main] elements", count);
        }

        if let Ok(sel) = Selector::parse("div[jsname]") {
            let count = doc.select(&sel).count();
            info!("Found {} div[jsname] elements", count);
        }

        if let Ok(sel) = Selector::parse("a[href*=\"url?\"]") {
            let count = doc.select(&sel).count();
            info!("Found {} search result links", count);
        }

        return Err(ScraperError::Parse(
            "No results found with current selectors".to_string(),
        ));
    }

    let title_selector = Selector::parse("h3")
        .map_err(|e| ScraperError::Parse(format!("Failed to parse title selector: {}", e)))?;
    let link_selector = Selector::parse("a")
        .map_err(|e| ScraperError::Parse(format!("Failed to parse link selector: {}", e)))?;

    let results = doc
        .select(&results_selector)
        .filter_map(|result| {
            let title_elem = result.select(&title_selector).next()?;
            let title = title_elem.inner_html();

            let link_elem = result.select(&link_selector).next()?;
            let link = link_elem.value().attr("href")?.to_string();

            Some(SearchResult { title, link })
        })
        .collect::<Vec<_>>();

    info!("Successfully parsed {} search results", results.len());
    Ok(results)
}
