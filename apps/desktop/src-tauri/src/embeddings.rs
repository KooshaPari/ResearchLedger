use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct OllamaEmbedder {
    pub endpoint: String,
    pub model: String,
}

impl OllamaEmbedder {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            endpoint: "http://127.0.0.1:11434/api/embed".into(),
            model: model.into(),
        }
    }
    pub async fn embed_batch(&self, inputs: &[String]) -> Result<Vec<Vec<f32>>, String> {
        let response = reqwest::Client::new()
            .post(&self.endpoint)
            .json(&serde_json::json!({ "model": self.model, "input": inputs }))
            .send()
            .await
            .map_err(|error| format!("local embedding service unavailable: {error}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "local embedding service returned {}",
                response.status()
            ));
        }
        Ok(response
            .json::<EmbedResponse>()
            .await
            .map_err(|error| error.to_string())?
            .embeddings)
    }
}

/// A score returned by an OpenAI-compatible local `/v1/rerank` endpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossEncoderScore {
    pub index: usize,
    pub relevance_score: f32,
}

#[derive(Debug, Clone)]
pub struct LocalCrossEncoder {
    endpoint: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct RerankRequest<'a> {
    model: &'a str,
    query: &'a str,
    documents: &'a [String],
}

#[derive(Debug, Deserialize)]
struct RerankResponse {
    results: Vec<RerankResponseItem>,
}

#[derive(Debug, Deserialize)]
struct RerankResponseItem {
    index: usize,
    relevance_score: f32,
}

impl LocalCrossEncoder {
    /// Creates a reranker that is strictly local. This prevents a user query or vault text
    /// from leaving the machine through a misconfigured endpoint.
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Result<Self, String> {
        let endpoint = endpoint.into();
        let url = url::Url::parse(&endpoint)
            .map_err(|error| format!("invalid rerank endpoint: {error}"))?;
        let local_host = matches!(
            url.host_str(),
            Some("localhost") | Some("127.0.0.1") | Some("::1")
        );
        if url.scheme() != "http" || !local_host {
            return Err("rerank endpoint must be an http loopback URL".into());
        }
        let model = model.into();
        if model.trim().is_empty() {
            return Err("rerank model must not be empty".into());
        }
        Ok(Self { endpoint, model })
    }

    /// Reranking is opt-in so an offline vault never causes a model download or a connection.
    /// `llama-server --rerank` exposes the OpenAI-compatible endpoint this adapter expects.
    pub fn from_environment() -> Result<Option<Self>, String> {
        let Ok(endpoint) = std::env::var("RESEARCHLEDGER_RERANK_ENDPOINT") else {
            return Ok(None);
        };
        let model = std::env::var("RESEARCHLEDGER_RERANK_MODEL")
            .unwrap_or_else(|_| "bge-reranker-v2-m3-q8".into());
        Self::new(endpoint, model).map(Some)
    }

    pub async fn rerank(
        &self,
        query: &str,
        documents: &[String],
    ) -> Result<Vec<CrossEncoderScore>, String> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        let response = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|error| format!("could not initialize local reranker: {error}"))?
            .post(&self.endpoint)
            .json(&RerankRequest {
                model: &self.model,
                query,
                documents,
            })
            .send()
            .await
            .map_err(|error| format!("local reranker unavailable: {error}"))?
            .error_for_status()
            .map_err(|error| format!("local reranker returned an error: {error}"))?
            .text()
            .await
            .map_err(|error| format!("could not read local reranker response: {error}"))?;
        Self::parse_response(&response, documents.len())
    }

    pub fn parse_response(
        body: &str,
        document_count: usize,
    ) -> Result<Vec<CrossEncoderScore>, String> {
        let response: RerankResponse = serde_json::from_str(body)
            .map_err(|error| format!("invalid local reranker response: {error}"))?;
        let mut seen = HashSet::new();
        let mut scores = Vec::with_capacity(response.results.len());
        for item in response.results {
            if item.index >= document_count {
                return Err("local reranker returned an out-of-range document index".into());
            }
            if !item.relevance_score.is_finite() {
                return Err("local reranker returned a non-finite relevance score".into());
            }
            if !seen.insert(item.index) {
                return Err("local reranker returned a duplicate document index".into());
            }
            scores.push(CrossEncoderScore {
                index: item.index,
                relevance_score: item.relevance_score,
            });
        }
        scores.sort_by(|left, right| {
            right
                .relevance_score
                .total_cmp(&left.relevance_score)
                .then(left.index.cmp(&right.index))
        });
        Ok(scores)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_local_ollama_endpoint() {
        let provider = OllamaEmbedder::new("embeddinggemma");
        assert_eq!(provider.endpoint, "http://127.0.0.1:11434/api/embed");
        assert_eq!(provider.model, "embeddinggemma");
    }

    #[test]
    fn cross_encoder_contract_fixture_preserves_provider_order() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/retrieval/cross_encoder_contract.json"
        ))
        .unwrap();
        let document_count = fixture["documents"].as_array().unwrap().len();
        let response = fixture["response"].to_string();

        let scores = LocalCrossEncoder::parse_response(&response, document_count).unwrap();
        let actual = scores
            .into_iter()
            .map(|score| score.index)
            .collect::<Vec<_>>();
        let expected = fixture["expected_order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as usize)
            .collect::<Vec<_>>();

        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_encoder_rejects_a_remote_endpoint() {
        assert!(LocalCrossEncoder::new("https://rerank.example.com/v1/rerank", "model").is_err());
    }

    #[test]
    fn selects_mlx_for_macos_and_tei_elsewhere() {
        assert_eq!(RerankEngine::default_for_target("macos"), RerankEngine::Mlx);
        assert_eq!(RerankEngine::default_for_target("linux"), RerankEngine::Tei);
        assert_eq!(RerankEngine::default_for_target("windows"), RerankEngine::Tei);
    }

    #[test]
    fn mlx_adapter_keeps_the_cohere_v1_rerank_contract() {
        let reranker = LocalCrossEncoder::new_with_engine(
            RerankEngine::Mlx,
            "http://127.0.0.1:9000/v1/rerank",
            "local-qwen-reranker",
        )
        .unwrap();

        assert_eq!(reranker.engine(), RerankEngine::Mlx);
        assert_eq!(reranker.protocol(), RerankProtocol::CohereV1);
    }
}
