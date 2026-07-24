//! Shared HTML parsing primitives for the third-party-provider connectors
//! (LinkedIn, Reddit, X). Both the per-provider parsers and tests reuse these
//! helpers to keep dedup-by-canonical-URL logic consistent and to avoid
//! copy-pasted boilerplate across modules.

use scraper::{element_ref::ElementRef, ElementRef as ScraperElementRef};

/// Normalise a raw `href` extracted from a timeline/saved-posts DOM.
///
/// * strips any query string and trailing slash;
/// * if `must_contain` is non-empty, requires the cleaned href to contain it
///   (e.g. `"/comments/"` or `"/status/"` for Reddit / X);
/// * if the href is relative, prefixes `base_url` (e.g. `"https://www.reddit.com"`).
///
/// Returns `Some(cleaned)` when the cleaned href passes the substring check, else `None`.
pub fn clean_post_href(raw_href: &str, must_contain: &str, base_url: &str) -> Option<String> {
    let cleaned = raw_href
        .split('?')
        .next()
        .unwrap_or(raw_href)
        .trim_end_matches('/');
    if cleaned.is_empty() {
        return None;
    }
    if !must_contain.is_empty() && !cleaned.contains(must_contain) {
        return None;
    }
    if cleaned.starts_with("http://") || cleaned.starts_with("https://") {
        Some(cleaned.to_string())
    } else if cleaned.starts_with('/') {
        Some(format!("{base_url}{cleaned}"))
    } else {
        Some(format!("{base_url}/{cleaned}"))
    }
}

/// Walk ancestors of `link` looking for the closest <article>-like container
/// whose collapsed text falls between `min_len` and `max_len` (inclusive
/// bounds). Whitespace is normalised to single spaces.
pub fn ancestor_text(link: ScraperElementRef<'_>, min_len: usize, max_len: usize) -> Option<String> {
    link.ancestors().find_map(|ancestor| {
        let element = ElementRef::wrap(ancestor)?;
        let value = element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        (value.len() >= min_len && value.len() <= max_len).then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::{Html, Selector};

    #[test]
    fn cleans_absolute_and_relative_hrefs() {
        assert_eq!(
            clean_post_href("https://x.com/u/status/123?lang=en", "/status/", "https://x.com"),
            Some("https://x.com/u/status/123".to_string())
        );
        assert_eq!(
            clean_post_href("/r/rust/comments/abc/hi/", "/comments/", "https://www.reddit.com"),
            Some("https://www.reddit.com/r/rust/comments/abc/hi".to_string())
        );
        assert_eq!(
            clean_post_href("u/status/9", "/status/", "https://x.com"),
            Some("https://x.com/u/status/9".to_string())
        );
    }

    #[test]
    fn rejects_when_substring_missing() {
        assert!(clean_post_href("https://x.com/about", "/status/", "https://x.com").is_none());
    }

    #[test]
    fn ancestor_text_walks_to_first_qualified_container() {
        let html = r#"<article><a href="/u/status/1">permalink</a><p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor.</p></article>"#;
        let document = Html::parse_document(html);
        let selector = Selector::parse("a[href*='/status/']").unwrap();
        let link = document.select(&selector).next().unwrap();
        let text = ancestor_text(link, 40, 20_000).expect("text should be extracted");
        assert!(text.contains("Lorem ipsum"));
    }
}
