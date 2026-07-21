use crate::{enrichment, storage::SourceDocument};

pub fn render_deterministic(document: &SourceDocument) -> String {
    let summary = document
        .content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("---") && !line.starts_with('#'))
        .unwrap_or("No summary available.");
    let summary = summary.chars().take(280).collect::<String>();
    let links = enrichment::extract_urls(&document.content);
    let references = if links.is_empty() {
        "- No outbound references discovered.".to_string()
    } else {
        links
            .iter()
            .map(|url| format!("- [{}]({})", url, enrichment::canonical_url(url)))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "---\ntype: Distilled Research Note\ntitle: {} — deterministic distillation\ndescription: Deterministic source summary and reference inventory\nresource: {}\ntags: [distillation, deterministic]\ntimestamp: {}\n---\n\n# Summary\n\n{}\n\n# Discovered References\n\n{}\n\n# Source\n\n[Original source]({})\n\n# Citations\n\n[1] [Original source]({})\n",
        document.title,
        document.source_uri.as_deref().unwrap_or("urn:researchledger:local"),
        document.captured_at,
        summary,
        references,
        document.source_uri.as_deref().unwrap_or("urn:researchledger:local"),
        document.source_uri.as_deref().unwrap_or("urn:researchledger:local")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_stable_summary_and_references() {
        let document = SourceDocument {
            id: "x".into(),
            relative_path: "x.md".into(),
            title: "Example".into(),
            source_kind: "test".into(),
            source_uri: Some("https://example.com/source".into()),
            content: "A concise research finding.\n\nSee https://example.com/reference.".into(),
            captured_at: "2026-01-01T00:00:00Z".into(),
        };
        let note = render_deterministic(&document);
        assert!(note.contains("type: Distilled Research Note"));
        assert!(note.contains("A concise research finding."));
        assert!(note.contains("https://example.com/reference"));
    }
}
