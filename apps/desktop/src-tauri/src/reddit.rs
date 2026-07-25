use scraper::{Html, Selector};
use std::collections::BTreeMap;

use crate::provider_html::{ancestor_text, clean_post_href, is_reddit_post_path};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RedditSavedPost {
    pub url: String,
    pub title: String,
    pub text: String,
    pub subreddit: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RedditCaptureFile {
    pub posts: Vec<RedditSavedPost>,
}

pub fn parse_capture_json(json: &str) -> Result<Vec<RedditSavedPost>, serde_json::Error> {
    Ok(serde_json::from_str::<RedditCaptureFile>(json)?.posts)
}

/// Parse Reddit's "Saved" page HTML. Reddit renders saved items as anchor tags on
/// post permalinks (`/r/<sub>/comments/<id>/...`). We collect title text from the
/// nearest heading and body text from the closest post container.
pub fn parse_saved_html(html: &str) -> Vec<RedditSavedPost> {
    let document = Html::parse_document(html);
    let link_selector =
        Selector::parse("a[href*='/comments/']").expect("static selector parses");
    let heading_selector = Selector::parse("h3, h2").expect("static selector parses");
    let mut posts = BTreeMap::new();
    for link in document.select(&link_selector) {
        let Some(raw_href) = link.value().attr("href") else { continue };
        let Some(url) = clean_post_href(raw_href, "/comments/", "https://www.reddit.com") else { continue };
        if !is_reddit_post_path(&url) { continue };
        let title = link
            .ancestors()
            .find_map(|ancestor| {
                let element = scraper::ElementRef::wrap(ancestor)?;
                element
                    .select(&heading_selector)
                    .next()
                    .map(|node| node.text().collect::<Vec<_>>().join(" ").split_whitespace().collect::<Vec<_>>().join(" "))
            })
            .unwrap_or_default();
        let text = match ancestor_text(link, 40, 20_000) {
            Some(value) if !value.is_empty() => value,
            _ => continue,
        };
        let subreddit = url
            .split("/r/")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
            .map(|value| value.to_string());
        posts
            .entry(url.clone())
            .or_insert(RedditSavedPost { url, title, text, subreddit });
    }
    posts.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_post_urls_and_text() {
        let html = r#"<main><article><a href="/r/rust/comments/abc123/why_local_first/">title</a><h3>Why local first?</h3><p>This is a deeply thoughtful post about local-first research ledgers and their tradeoffs.</p></article><a href="/r/rust/comments/abc123/why_local_first/">again</a></main>"#;
        let posts = parse_saved_html(html);
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].url, "https://www.reddit.com/r/rust/comments/abc123/why_local_first");
        assert!(posts[0].text.contains("local-first research ledgers"));
        assert_eq!(posts[0].subreddit.as_deref(), Some("rust"));
    }

    #[test]
    fn rejects_user_comment_activity_urls() {
        // Reddit users have a /user/<name>/comments/ activity feed that ALSO matches
        // /comments/ anchors. The capture path must distinguish post permalinks from
        // user-profile activity.
        let html = r#"<main>
            <article><a href="/user/koosha/comments/abc/why_local_first/">user activity</a><p>noise</p></article>
            <article><a href="/r/rust/comments/abc/why_local_first/">post permalink</a><p>A thoughtful post about local-first research ledgers and the tradeoffs of owning your data.</p></article>
        </main>"#;
        let posts = parse_saved_html(html);
        assert_eq!(posts.len(), 1, "user-comment activity must be filtered; got {:?}", posts.iter().map(|p| &p.url).collect::<Vec<_>>());
        assert!(posts[0].url.contains("/r/rust/comments/"));
    }

    #[test]
    fn parses_playwright_capture_file() {
        let posts = parse_capture_json(r#"{"version":1,"posts":[{"url":"https://www.reddit.com/r/rust/comments/abc/hi/","title":"hi","text":"A thoughtful saved post about Rust research tooling.","subreddit":"rust"}]}"#).unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].subreddit.as_deref(), Some("rust"));
        assert!(posts[0].text.starts_with("A thoughtful"));
    }
}
