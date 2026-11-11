#![cfg(feature = "web")]

use scraper::{Html, Selector};

/// Extract a document title and visible-ish body text from an HTML response.
pub fn extract_text(html: &str) -> (String, String) {
    let document = Html::parse_document(html);
    let title_selector = Selector::parse("title").expect("static selector");
    let title = document.select(&title_selector).next()
        .map(|node| node.text().collect::<String>())
        .unwrap_or_default()
        .trim()
        .to_owned();

    for selector_text in ["article", "main", "body"] {
        let selector = Selector::parse(selector_text).expect("static selector");
        if let Some(node) = document.select(&selector).next() {
            let text = node.text().collect::<Vec<_>>().join(" ");
            let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !cleaned.is_empty() { return (title, cleaned); }
        }
    }
    (title, String::new())
}
