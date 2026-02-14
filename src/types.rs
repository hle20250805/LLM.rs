use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InputText {
    Single(String),
    Multiple(Vec<String>),
}

impl InputText {
    pub fn into_vec(self) -> Vec<String> {
        match self {
            InputText::Single(s) => vec![s],
            InputText::Multiple(v) => v,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenAIEmbeddingsRequest {
    pub input: InputText,
    pub model: Option<String>,
    pub encoding_format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIEmbeddingsResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmbedRequest {
    pub texts: InputText,
    pub normalize_embeddings: bool,
    pub batch_size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmbedResponse {
    pub vectors: Vec<Vec<f32>>,
    pub count: usize,
    pub vector_dim: usize,
    pub model_path: String,
}
