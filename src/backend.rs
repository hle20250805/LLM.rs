use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::types::{EmbedRequest, EmbedResponse, InputText};

pub mod candle;



#[async_trait]
pub trait EmbeddingBackend: Send + Sync {
    async fn embed(
        &self,
        texts: Vec<String>,
        normalize_embeddings: bool,
        batch_size: u32,
    ) -> Result<EmbedResponse, BackendError>;
}

#[derive(Clone)]
pub struct BackendClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("backend request failed: {0}")]
    Request(String),
    #[error("backend returned non-success status: {0}")]
    Status(StatusCode),
    #[error("backend decode failed: {0}")]
    Decode(String),
}

impl BackendClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn embed(
        &self,
        texts: Vec<String>,
        normalize_embeddings: bool,
        batch_size: u32,
    ) -> Result<EmbedResponse, BackendError> {
        let url = format!("{}/embed", self.base_url.trim_end_matches('/'));
        let payload = EmbedRequest {
            texts: InputText::Multiple(texts),
            normalize_embeddings,
            batch_size,
        };

        let res = self
            .client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| BackendError::Request(e.to_string()))?;

        if !res.status().is_success() {
            return Err(BackendError::Status(res.status()));
        }

        res.json::<EmbedResponse>()
            .await
            .map_err(|e| BackendError::Decode(e.to_string()))
    }
}

#[async_trait]
impl EmbeddingBackend for BackendClient {
    async fn embed(
        &self,
        texts: Vec<String>,
        normalize_embeddings: bool,
        batch_size: u32,
    ) -> Result<EmbedResponse, BackendError> {
        self.embed(texts, normalize_embeddings, batch_size).await
    }
}

// 后端类型枚举
#[derive(Debug, Clone, serde::Deserialize)]
pub enum BackendType {
    Proxy,
    Candle,
}

impl Default for BackendType {
    fn default() -> Self {
        Self::Proxy
    }
}

impl std::str::FromStr for BackendType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proxy" => Ok(Self::Proxy),
            "candle" => Ok(Self::Candle),
            _ => Err(format!("Invalid backend type: {}", s)),
        }
    }
}
