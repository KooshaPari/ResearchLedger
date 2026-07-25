use scraper::{Html, Selector};
use std::collections::BTreeMap;

use crate::provider_html::{ancestor_text, clean_post_href, is_x_post_path};

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
        let Some(raw_href) = link.value().attr("href") else { continue };
        let Some(url) = clean_post_href(raw_href, "/status/", "https://x.com") else { continue };
        if !is_x_post_path(&url) {
            continue;
        }
        let text = match ancestor_text(link, 40, 20_000) {
            Some(value) if !value.is_empty() => value,
            _ => continue,
        };
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

    #[test]
    fn rejects_non_status_permalink_shapes() {
        // /i/status/... is X-internal share format — not a real permalink.
        let html = r#"<article><a href="/i/status/1100000000000000099">share</a><div><span>@someone</span><p>noise that looks like a real post but isn't a permalink anchor.</p></div></article>"#;
        let posts = parse_bookmarks_html(html);
        assert!(
            posts.is_empty(),
            "expected zero posts; got {posts:?}"
        );

        // /intent/... (web intent URLs) must be filtered out.
        let html = r#"<article><a href="/intent/like?tweet_id=1100000000000000099">like</a><div><span>@someone</span><p>intent URL noise that must not slip into the bookmarks vault.</p></div></article>"#;
        let posts = parse_bookmarks_html(html);
        assert!(posts.is_empty());

        // /messages/... is a DM thread, not a bookmarked post.
        let html = r#"<article><a href="/messages/1234-5678">DM</a><div><span>@someone</span><p>DM thread noise that must not slip into the bookmarks vault.</p></div></article>"#;
        let posts = parse_bookmarks_html(html);
        assert!(posts.is_empty());
    }
}
