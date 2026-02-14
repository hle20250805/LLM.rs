mod backend;
mod config;
mod queue;
mod types;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use thiserror::Error;
use tracing::info;

use std::sync::Arc;

use crate::backend::{BackendClient, BackendType, EmbeddingBackend};
use crate::config::Config;
use crate::queue::Queue;
use crate::types::{
    EmbedRequest, EmbedResponse, EmbeddingData, OpenAIEmbeddingsRequest,
    OpenAIEmbeddingsResponse, Usage,
};

#[derive(Clone)]
struct AppState {
    queue: Queue,
    config: Config,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("backend error: {0}")]
    Backend(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Backend(msg) => (StatusCode::BAD_GATEWAY, msg),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let config = Config::from_env_or_file();
    
    // 根据配置选择后端
    let backend: Arc<dyn EmbeddingBackend> = match config.backend_type {
        BackendType::Proxy => {
            info!("Using proxy backend: {}", config.backend_url);
            Arc::new(BackendClient::new(config.backend_url.clone()))
        }
        BackendType::Candle => {
            info!("Using candle backend: {}", config.model_path);
            let candle_backend = crate::backend::candle::CandleBackend::new(config.model_path.clone())
                .expect("Failed to create candle backend");
            Arc::new(candle_backend)
        }
    };
    
    let queue = Queue::new(backend, config.workers, config.queue_capacity);

    let host = config.host.clone();
    let port = config.port;
    let state = AppState { queue, config };

    let app = Router::new()
        .route("/health", get(health))
        .route("/embed", post(embed_compat))
        .route("/v1/embeddings", post(openai_embeddings))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    info!("LLM.rs listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "backend_url": state.config.backend_url,
        "model_name": state.config.model_name,
    }))
}

async fn openai_embeddings(
    State(state): State<AppState>,
    Json(payload): Json<OpenAIEmbeddingsRequest>,
) -> Result<Json<OpenAIEmbeddingsResponse>, AppError> {
    let texts = payload.input.into_vec();
    if texts.is_empty() {
        return Err(AppError::BadRequest("input cannot be empty".to_string()));
    }

    if let Some(format) = payload.encoding_format.as_deref() {
        if format != "float" {
            return Err(AppError::BadRequest(
                "only encoding_format=float is supported".to_string(),
            ));
        }
    }

    let response = state
        .queue
        .enqueue(
            texts,
            state.config.normalize_embeddings,
            state.config.batch_size,
        )
        .await
        .map_err(|e| AppError::Backend(e.to_string()))?;

    Ok(Json(map_openai_response(
        payload.model.unwrap_or_else(|| state.config.model_name.clone()),
        response,
    )))
}

async fn embed_compat(
    State(state): State<AppState>,
    Json(payload): Json<EmbedRequest>,
) -> Result<Json<EmbedResponse>, AppError> {
    let texts = payload.texts.into_vec();
    if texts.is_empty() {
        return Err(AppError::BadRequest("texts cannot be empty".to_string()));
    }

    let response = state
        .queue
        .enqueue(texts, payload.normalize_embeddings, payload.batch_size)
        .await
        .map_err(|e| AppError::Backend(e.to_string()))?;

    Ok(Json(response))
}

fn map_openai_response(model: String, embed: EmbedResponse) -> OpenAIEmbeddingsResponse {
    let data = embed
        .vectors
        .into_iter()
        .enumerate()
        .map(|(index, embedding)| EmbeddingData {
            object: "embedding".to_string(),
            embedding,
            index,
        })
        .collect();

    OpenAIEmbeddingsResponse {
        object: "list".to_string(),
        data,
        model,
        usage: Usage {
            prompt_tokens: 0,
            total_tokens: 0,
        },
    }
}
