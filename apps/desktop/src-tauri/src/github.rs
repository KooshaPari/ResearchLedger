use base64::Engine;
use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StarredRepository {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub default_branch: String,
    pub language: Option<String>,
    pub topics: Vec<String>,
    pub owner: RepositoryOwner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryOwner {
    pub login: String,
}

#[derive(Debug, Deserialize)]
struct ReadmeResponse {
    content: String,
    encoding: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAuthorization {
    #[serde(alias = "device_code")]
    pub device_code: String,
    #[serde(alias = "user_code")]
    pub user_code: String,
    #[serde(alias = "verification_uri")]
    pub verification_uri: String,
    #[serde(alias = "expires_in")]
    pub expires_in: u64,
    #[serde(alias = "interval")]
    pub interval: u64,
}

#[derive(Debug, serde::Deserialize)]
struct DeviceTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

const DEVICE_FLOW_SCOPE: &str = "public_repo";

fn device_flow_scope() -> &'static str {
    DEVICE_FLOW_SCOPE
}

#[derive(Debug)]
pub enum GithubError {
    Http(reqwest::Error),
    RateLimited,
    InvalidResponse(String),
    Decode(String),
}

impl std::fmt::Display for GithubError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(error) => write!(formatter, "GitHub request failed: {error}"),
            Self::RateLimited => write!(formatter, "GitHub rate limit exceeded"),
            Self::InvalidResponse(status) => write!(formatter, "GitHub returned {status}"),
            Self::Decode(message) => write!(formatter, "GitHub README decode failed: {message}"),
        }
    }
}

impl std::error::Error for GithubError {}

pub struct GithubClient {
    client: Client,
    token: String,
}

impl GithubClient {
    pub fn new(token: impl Into<String>) -> Result<Self, GithubError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "x-github-api-version",
            header::HeaderValue::from_static("2022-11-28"),
        );
        let client = Client::builder()
            .default_headers(headers)
            .user_agent("ResearchLedger/0.1")
            .build()
            .map_err(GithubError::Http)?;
        Ok(Self {
            client,
            token: token.into(),
        })
    }

    pub async fn list_starred(&self) -> Result<Vec<StarredRepository>, GithubError> {
        let mut page = 1;
        let mut repositories = Vec::new();
        loop {
            let response = self
                .client
                .get("https://api.github.com/user/starred")
                .bearer_auth(&self.token)
                .query(&[("per_page", 100), ("page", page)])
                .send()
                .await
                .map_err(GithubError::Http)?;
            if response.status() == StatusCode::FORBIDDEN {
                return Err(GithubError::RateLimited);
            }
            if !response.status().is_success() {
                return Err(GithubError::InvalidResponse(response.status().to_string()));
            }
            let batch: Vec<StarredRepository> = response.json().await.map_err(GithubError::Http)?;
            let done = batch.len() < 100;
            repositories.extend(batch);
            if done {
                return Ok(repositories);
            }
            page += 1;
        }
    }

    pub async fn read_readme(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<String>, GithubError> {
        let response = self
            .client
            .get(format!(
                "https://api.github.com/repos/{owner}/{repo}/readme"
            ))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(GithubError::Http)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if response.status() == StatusCode::FORBIDDEN {
            return Err(GithubError::RateLimited);
        }
        if !response.status().is_success() {
            return Err(GithubError::InvalidResponse(response.status().to_string()));
        }
        let payload: ReadmeResponse = response.json().await.map_err(GithubError::Http)?;
        if payload.encoding != "base64" {
            return Err(GithubError::Decode(format!(
                "unsupported README encoding: {}",
                payload.encoding
            )));
        }
        let normalized = payload.content.replace('\n', "");
        base64::engine::general_purpose::STANDARD
            .decode(normalized)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .map(Some)
            .map_err(|error| GithubError::Decode(error.to_string()))
    }

    pub async fn request_device_authorization(
        &self,
        client_id: &str,
    ) -> Result<DeviceAuthorization, GithubError> {
        let response = self
            .client
            .post("https://github.com/login/device/code")
            // GitHub returns form-encoded data unless this endpoint is
            // explicitly negotiated as JSON. The shared API default is not
            // sufficient for the OAuth host.
            .header(header::ACCEPT, "application/json")
            .form(&[("client_id", client_id), ("scope", device_flow_scope())])
            .send()
            .await
            .map_err(GithubError::Http)?;
        if !response.status().is_success() {
            let status = response.status();
            let detail = response.text().await.unwrap_or_default();
            return Err(GithubError::InvalidResponse(format!(
                "{status}: {detail}"
            )));
        }
        response.json().await.map_err(GithubError::Http)
    }

    pub async fn poll_device_token(
        &self,
        client_id: &str,
        device_code: &str,
        interval: u64,
        expires_in: u64,
    ) -> Result<String, GithubError> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(expires_in);
        let mut delay = interval.max(5);
        while std::time::Instant::now() < deadline {
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            let response = self
                .client
                .post("https://github.com/login/oauth/access_token")
                .header(header::ACCEPT, "application/json")
                .form(&[
                    ("client_id", client_id),
                    ("device_code", device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await
                .map_err(GithubError::Http)?;
            let status = response.status();
            let payload: DeviceTokenResponse = response.json().await.map_err(GithubError::Http)?;
            if let Some(token) = payload.access_token {
                return Ok(token);
            }
            if payload.error.as_deref() == Some("slow_down") {
                delay += 5;
            }
            if payload.error.as_deref() != Some("authorization_pending")
                && payload.error.as_deref() != Some("slow_down")
            {
                let error = payload.error.unwrap_or_else(|| "authorization failed".into());
                let description = payload.error_description.unwrap_or_default();
                let detail = if description.is_empty() {
                    error
                } else {
                    format!("{error}: {description}")
                };
                return Err(GithubError::InvalidResponse(format!("{status}: {detail}")));
            }
        }
        Err(GithubError::InvalidResponse(
            "device authorization expired".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_github_readme_payload() {
        let value = base64::engine::general_purpose::STANDARD.encode("# Hello\n");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(value)
            .unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "# Hello\n");
    }

    #[test]
    fn device_flow_requests_scope_that_allows_starred_repositories() {
        assert_eq!(device_flow_scope(), "public_repo");
    }

    #[test]
    fn decodes_github_snake_case_device_authorization_json() {
        let value: DeviceAuthorization = serde_json::from_str(
            r#"{"device_code":"device","user_code":"ABCD-1234","verification_uri":"https://github.com/login/device","expires_in":600,"interval":5}"#,
        )
        .unwrap();
        assert_eq!(value.verification_uri, "https://github.com/login/device");
        assert_eq!(value.expires_in, 600);
    }

    #[test]
    fn decodes_device_error_description() {
        let value: DeviceTokenResponse = serde_json::from_str(
            r#"{"error":"authorization_pending","error_description":"The user has not yet completed"}"#,
        )
        .unwrap();
        assert_eq!(value.error.as_deref(), Some("authorization_pending"));
        assert!(value.error_description.unwrap().contains("not yet"));
    }
}
