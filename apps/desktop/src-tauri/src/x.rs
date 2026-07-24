use scraper::{Html, Selector};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct XSavedPost {
    pub url: String,
    pub author: String,
    pub text: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct XCaptureFile {
    pub posts: Vec<XSavedPost>,
}

pub fn parse_capture_json(json: &str) -> Result<Vec<XSavedPost>, serde_json::Error> {
    Ok(serde_json::from_str::<XCaptureFile>(json)?.posts)
}

/// Parse the X "Bookmarks" page. Each bookmark in the modern X timeline is an
/// anchor with an `/status/<id>` permalink. We collect author handle from the
/// nearest `@user` text and body from the closest article container.
pub fn parse_bookmarks_html(html: &str) -> Vec<XSavedPost> {
    let document = Html::parse_document(html);
    let status_selector =
        Selector::parse("a[href*='/status/']").expect("static selector parses");
    let mut posts = BTreeMap::new();
    for link in document.select(&status_selector) {
        let Some(raw_href) = link.value().attr("href") else {
            continue;
        };
        let cleaned = raw_href.split('?').next().unwrap_or(raw_href).trim_end_matches('/');
        if !cleaned.contains("/status/") {
            continue;
        }
        let url = if cleaned.starts_with("http") {
            cleaned.to_string()
        } else {
            format!("https://x.com{cleaned}")
        };
        let text = link
            .ancestors()
            .find_map(|ancestor| {
                let element = scraper::ElementRef::wrap(ancestor)?;
                let value = element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                (value.len() > 40 && value.len() < 20_000).then_some(value)
            })
            .unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        let author = text
            .split('@')
            .nth(1)
            .and_then(|rest| {
                rest.chars()
                    .take_while(|character| character.is_alphanumeric() || *character == '_')
                    .collect::<String>()
                    .into()
            })
            .unwrap_or_default();
        posts
            .entry(url.clone())
            .or_insert(XSavedPost { url, author, text });
    }
    posts.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_status_urls_and_text() {
        let html = r#"<article><a href="/someone/status/1100000000000000001">permalink</a><div><span>@someone</span><p>This is a thoughtful bookmark about Rust research ledgers and durable provenance.</p></div></article><a href="/someone/status/1100000000000000001">again</a></article>"#;
        let posts = parse_bookmarks_html(html);
        assert_eq!(posts.len(), 1);
        assert_eq!(
            posts[0].url,
            "https://x.com/someone/status/1100000000000000001"
        );
        assert_eq!(posts[0].author, "someone");
        assert!(posts[0].text.contains("durable provenance"));
    }

    #[test]
    fn parses_playwright_capture_file() {
        let posts = parse_capture_json(r#"{"version":1,"posts":[{"url":"https://x.com/someone/status/42","author":"someone","text":"A thoughtful bookmark about local-first research tooling and provenance."}]}"#).unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].author, "someone");
        assert!(posts[0].text.starts_with("A thoughtful"));
    }
}
