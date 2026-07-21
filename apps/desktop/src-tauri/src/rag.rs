use crate::storage::SearchResult;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct VectorHit {
    pub document_id: String,
    pub score: f32,
}

/// Reciprocal-rank fusion seam for a future local/cloud vector provider.
/// Lexical FTS5 remains authoritative when no vector hits are supplied.
pub fn fuse_ranked(
    lexical: Vec<SearchResult>,
    vector: Vec<VectorHit>,
    limit: usize,
) -> Vec<SearchResult> {
    let mut scores: HashMap<String, (f32, SearchResult)> = HashMap::new();
    for (rank, result) in lexical.into_iter().enumerate() {
        let score = 1.0 / (60.0 + rank as f32 + 1.0);
        scores
            .entry(result.document_id.clone())
            .or_insert((0.0, result))
            .0 += score;
    }
    for (rank, hit) in vector.into_iter().enumerate() {
        if let Some((score, _)) = scores.get_mut(&hit.document_id) {
            *score += hit.score.max(0.0) / (60.0 + rank as f32 + 1.0);
        }
    }
    let mut ranked = scores.into_values().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.0.total_cmp(&left.0));
    ranked
        .into_iter()
        .take(limit)
        .map(|(_, result)| result)
        .collect()
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

    #[test]
    fn lexical_results_remain_available_without_vectors() {
        let result = SearchResult {
            document_id: "doc".into(),
            title: "Doc".into(),
            source_uri: None,
            snippet: "text".into(),
        };
        assert_eq!(fuse_ranked(vec![result], vec![], 10).len(), 1);
    }
}
