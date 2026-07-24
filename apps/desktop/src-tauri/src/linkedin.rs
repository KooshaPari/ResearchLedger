use scraper::{Html, Selector};
use std::collections::BTreeMap;

use crate::provider_html::{ancestor_text, clean_post_href};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct LinkedInPost {
    pub url: String,
    pub text: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CaptureFile {
    pub posts: Vec<LinkedInPost>,
}

pub fn parse_capture_json(json: &str) -> Result<Vec<LinkedInPost>, serde_json::Error> {
    Ok(serde_json::from_str::<CaptureFile>(json)?.posts)
}

pub fn parse_activity_html(html: &str) -> Vec<LinkedInPost> {
    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a[href*='feed/update/urn:li:activity:']").unwrap();
    let article_selector = Selector::parse("article").unwrap();
    let mut posts = BTreeMap::new();
    for link in document.select(&link_selector) {
        let Some(raw_href) = link.value().attr("href") else { continue };
        let Some(url) = clean_post_href(raw_href, "feed/update/urn:li:activity:", "") else { continue };
        let text = ancestor_text(link, 40, 20_000).unwrap_or_else(|| {
            document
                .select(&article_selector)
                .next()
                .map(|node| node.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default()
        });
        posts.entry(url.clone()).or_insert(LinkedInPost { url, text });
    }
    posts.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_post_urls_and_text() {
        let html = r#"<main><article><a href="https://www.linkedin.com/feed/update/urn:li:activity:123/">Open</a><p>A useful post about local research systems and durable knowledge.</p></article><a href="https://www.linkedin.com/feed/update/urn:li:activity:123/">Again</a></main>"#;
        let posts = parse_activity_html(html);
        assert_eq!(posts.len(), 1);
        assert_eq!(
            posts[0].url,
            "https://www.linkedin.com/feed/update/urn:li:activity:123"
        );
        assert!(posts[0].text.contains("local research systems"));
    }

    #[test]
    fn parses_playwright_capture_file() {
        let posts = parse_capture_json(r#"{"version":1,"posts":[{"url":"https://www.linkedin.com/feed/update/urn:li:activity:42","text":"A captured research post with enough useful text."}]}"#).unwrap();
        assert_eq!(
            posts[0].url,
            "https://www.linkedin.com/feed/update/urn:li:activity:42"
        );
    }
}
