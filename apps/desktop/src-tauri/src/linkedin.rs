use scraper::{Html, Selector};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedInPost {
    pub url: String,
    pub text: String,
}

pub fn parse_activity_html(html: &str) -> Vec<LinkedInPost> {
    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a[href*='feed/update/urn:li:activity:']").unwrap();
    let article_selector = Selector::parse("article").unwrap();
    let mut posts = BTreeMap::new();
    for link in document.select(&link_selector) {
        let Some(url) = link.value().attr("href") else {
            continue;
        };
        let url = url
            .split('?')
            .next()
            .unwrap_or(url)
            .trim_end_matches('/')
            .to_string();
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
            .or_else(|| {
                document
                    .select(&article_selector)
                    .next()
                    .map(|node| node.text().collect::<Vec<_>>().join(" "))
            })
            .unwrap_or_default();
        posts
            .entry(url.clone())
            .or_insert(LinkedInPost { url, text });
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
}
