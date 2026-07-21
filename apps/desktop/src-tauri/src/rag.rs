use crate::storage::SearchResult;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub document_id: String,
    pub title: String,
    pub source_uri: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalContext {
    pub query: String,
    pub context: String,
    pub citations: Vec<Citation>,
}

pub fn build_context(query: &str, results: Vec<SearchResult>) -> RetrievalContext {
    let citations = results
        .into_iter()
        .map(|result| Citation {
            document_id: result.document_id,
            title: result.title,
            source_uri: result.source_uri,
            snippet: result.snippet,
        })
        .collect::<Vec<_>>();
    let context = citations
        .iter()
        .enumerate()
        .map(|(index, citation)| {
            format!("[{}] {}\n{}", index + 1, citation.title, citation.snippet)
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    RetrievalContext {
        query: query.into(),
        context,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_keeps_citations_aligned_with_snippets() {
        let bundle = build_context(
            "agents",
            vec![SearchResult {
                document_id: "github:octo/agents".into(),
                title: "Agents".into(),
                source_uri: Some("https://github.com/octo/agents".into()),
                snippet: "agent workflow".into(),
            }],
        );
        assert!(bundle.context.contains("[1] Agents"));
        assert_eq!(
            bundle.citations[0].source_uri.as_deref(),
            Some("https://github.com/octo/agents")
        );
    }
}
