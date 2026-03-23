use log::{info, warn};
use scraper::{Html, Selector};
use std::fs;
use urlencoding::decode;

use crate::Result;
use crate::error::ScraperError;

pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub content: Option<String>,
}

/// Extract text content from a selector element, collapsing whitespace
fn extract_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Remove Yahoo-specific tracking URL wrapper
fn parse_url(url_string: &str) -> String {
    let endings = ["/RS", "/RK"];
    let mut end_positions = Vec::new();

    // Find the start position of the actual URL after /RU=
    let ru_pos = match url_string.find("/RU=") {
        Some(pos) => pos + 4, // Skip past "/RU="
        None => return url_string.to_string(),
    };

    // Find the first "http" occurrence after /RU=
    let start = match url_string[ru_pos..].find("http") {
        Some(pos) => ru_pos + pos,
        None => return url_string.to_string(),
    };

    // Find all ending positions
    for ending in &endings {
        if let Some(pos) = url_string.rfind(ending) {
            end_positions.push(pos);
        }
    }

    // If no valid end positions found, return original
    if end_positions.is_empty() {
        return url_string.to_string();
    }

    // Get the minimum end position (the actual end of the URL)
    let end = *end_positions.iter().min().unwrap();

    // Extract and decode the URL
    let encoded_url = &url_string[start..end];
    decode(encoded_url)
        .map(|decoded| decoded.to_string())
        .unwrap_or_else(|_| encoded_url.to_string())
}

pub fn run(html: &str) -> Result<Vec<SearchResult>> {
    info!("Starting to parse HTML content");

    if html.is_empty() {
        warn!("Parse failed: HTML content is empty");
        return Err(ScraperError::Parse("HTML content is empty".to_string()));
    }

    info!("HTML length: {} bytes", html.len());

    let doc = Html::parse_document(html);

    // Pre-compile selectors once to avoid repeated parsing
    let result_selector = Selector::parse("div.algo-sr").expect("Invalid result selector");
    let primary_link_selector =
        Selector::parse("div.compTitle h3 a").expect("Invalid primary link selector");
    let fallback_link_selector =
        Selector::parse("div.compTitle a").expect("Invalid fallback link selector");
    let title_selector = Selector::parse("h3 a").expect("Invalid title selector");
    let title_span_selector =
        Selector::parse("div.compTitle a h3 span").expect("Invalid title span selector");
    let content_selector = Selector::parse("div.compText").expect("Invalid content selector");

    let results_iter: Vec<_> = doc.select(&result_selector).collect();

    if results_iter.is_empty() {
        warn!("No results found with selector 'div.algo-sr'");
        let _ = fs::write("parse_debug_html.html", html);
        info!("Saved HTML to parse_debug_html.html for inspection");
        return Err(ScraperError::Parse("No search results found".to_string()));
    }

    let mut results = Vec::new();

    for result_elem in results_iter {
        // Try to extract URL from div.compTitle > h3 > a, fallback to div.compTitle > a
        let link = result_elem
            .select(&primary_link_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .or_else(|| {
                result_elem
                    .select(&fallback_link_selector)
                    .next()
                    .and_then(|e| e.value().attr("href"))
            })
            .map(|s| s.to_string());

        // Skip if no URL found
        let link = match link {
            Some(url) => parse_url(&url),
            None => continue,
        };

        // Try to extract title from aria-label first, then from h3 span text
        let title = result_elem
            .select(&title_selector)
            .next()
            .and_then(|e| e.value().attr("aria-label"))
            .map(|s| s.to_string())
            .or_else(|| {
                result_elem
                    .select(&title_span_selector)
                    .next()
                    .map(|e| e.text().collect::<String>())
            })
            .unwrap_or_default();

        let title = extract_text(&title);

        // Extract content from div.compText
        let content = result_elem.select(&content_selector).next().map(|e| {
            let text = e.text().collect::<String>();
            extract_text(&text)
        });

        results.push(SearchResult {
            title,
            link,
            content,
        });
    }

    if results.is_empty() {
        warn!("Failed to parse any results from HTML");
        return Err(ScraperError::Parse(
            "Failed to parse search results".to_string(),
        ));
    }

    info!("Successfully parsed {} search results", results.len());
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_debug_html() {
        // Read the debug HTML file
        let html_content = fs::read_to_string("parse_debug_html.html")
            .expect("Failed to read parse_debug_html.html");

        // Parse the HTML
        let results = run(&html_content).expect("Failed to parse HTML");

        // Verify we got some results
        assert!(
            !results.is_empty(),
            "Expected to find search results in parse_debug_html.html"
        );

        // Verify each result has the required fields
        for result in results {
            assert!(!result.title.is_empty(), "Title should not be empty");
            assert!(!result.link.is_empty(), "Link should not be empty");

            println!(
                "Title: {}\nLink: {}\nContent: {:?}\n",
                result.title, result.link, result.content
            );
        }
    }

    #[test]
    fn test_parse_url_with_yahoo_wrapper() {
        let yahoo_url = "/RU=http://example.com/path/RS";
        let result = parse_url(yahoo_url);
        assert_eq!(result, "http://example.com/path");
    }

    #[test]
    fn test_parse_url_with_rk_ending() {
        let yahoo_url = "/RU=http://example.com/path/RK=123";
        let result = parse_url(yahoo_url);
        assert_eq!(result, "http://example.com/path");
    }

    #[test]
    fn test_parse_url_direct_url() {
        let direct_url = "http://example.com/path";
        let result = parse_url(direct_url);
        assert_eq!(result, "http://example.com/path");
    }

    #[test]
    fn test_extract_text_whitespace() {
        let text = "  This   is   a  test  ";
        let result = extract_text(text);
        assert_eq!(result, "This is a test");
    }
}
