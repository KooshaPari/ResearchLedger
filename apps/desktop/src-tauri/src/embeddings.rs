use serde::Deserialize;

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_to_local_ollama_endpoint() {
        let provider = OllamaEmbedder::new("embeddinggemma");
        assert_eq!(provider.endpoint, "http://127.0.0.1:11434/api/embed");
        assert_eq!(provider.model, "embeddinggemma");
    }
}
