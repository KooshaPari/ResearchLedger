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
    pub result: Option<SearchResult>,
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
        let contribution = hit.score.max(0.0) / (60.0 + rank as f32 + 1.0);
        if let Some((score, _)) = scores.get_mut(&hit.document_id) {
            *score += contribution;
        } else if let Some(result) = hit.result {
            scores.insert(hit.document_id, (contribution, result));
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

/// Deterministic reranker fallback used when no local cross-encoder is configured.
/// It rewards query-token overlap in the title and snippet while preserving the
/// fused order for ties, so retrieval remains explainable and offline-safe.
pub fn rerank(query: &str, results: Vec<SearchResult>) -> Vec<SearchResult> {
    let tokens = query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| token.len() > 2)
        .collect::<Vec<_>>();
    let mut ranked = results
        .into_iter()
        .enumerate()
        .map(|(index, result)| {
            let haystack = format!("{} {}", result.title, result.snippet).to_ascii_lowercase();
            let score = tokens
                .iter()
                .filter(|token| haystack.contains(token.as_str()))
                .count();
            (score, index, result)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    ranked.into_iter().map(|(_, _, result)| result).collect()
}

/// Applies verified cross-encoder scores and appends any unscored candidates in their fused
/// order. The latter keeps partial provider responses deterministic and citation-safe.
pub fn rerank_with_cross_encoder(
    results: Vec<SearchResult>,
    scores: Vec<crate::embeddings::CrossEncoderScore>,
) -> Vec<SearchResult> {
    let mut score_by_index = HashMap::new();
    for score in scores {
        score_by_index.insert(score.index, score.relevance_score);
    }
    let mut ranked = results
        .into_iter()
        .enumerate()
        .map(|(index, result)| (score_by_index.get(&index).copied(), index, result))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| match (left.0, right.0) {
        (Some(left_score), Some(right_score)) => right_score
            .total_cmp(&left_score)
            .then(left.1.cmp(&right.1)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.1.cmp(&right.1),
    });
    ranked.into_iter().map(|(_, _, result)| result).collect()
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

    #[test]
    fn vector_only_results_are_not_dropped() {
        let result = SearchResult {
            document_id: "vector-doc".into(),
            title: "Vector doc".into(),
            source_uri: None,
            snippet: "semantic match".into(),
        };
        let hits = vec![VectorHit {
            document_id: result.document_id.clone(),
            score: 0.9,
            result: Some(result),
        }];
        assert_eq!(fuse_ranked(vec![], hits, 10)[0].document_id, "vector-doc");
    }

    #[test]
    fn deterministic_reranker_preserves_ties() {
        let first = SearchResult {
            document_id: "first".into(),
            title: "Agents and ledgers".into(),
            source_uri: None,
            snippet: "local research".into(),
        };
        let second = SearchResult {
            document_id: "second".into(),
            title: "Other note".into(),
            source_uri: None,
            snippet: "unrelated".into(),
        };
        let ranked = rerank("agents", vec![second, first]);
        assert_eq!(ranked[0].document_id, "first");
    }

    #[test]
    fn cross_encoder_scores_reorder_the_fixture_candidates() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/retrieval/cross_encoder_contract.json"
        ))
        .unwrap();
        let scores = crate::embeddings::LocalCrossEncoder::parse_response(
            &fixture["response"].to_string(),
            fixture["documents"].as_array().unwrap().len(),
        )
        .unwrap();
        let results = fixture["documents"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(index, document)| SearchResult {
                document_id: index.to_string(),
                title: document.as_str().unwrap().to_string(),
                source_uri: None,
                snippet: document.as_str().unwrap().to_string(),
            })
            .collect();

        let ranked = rerank_with_cross_encoder(results, scores);
        assert_eq!(
            ranked
                .into_iter()
                .map(|result| result.document_id)
                .collect::<Vec<_>>(),
            ["2", "1", "0"]
        );
    }
}
