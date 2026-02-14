use serde::Deserialize;
use std::{env, fs, path::Path};

use crate::backend::BackendType;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub backend_url: String,
    pub backend_type: BackendType,
    pub model_path: String,
    pub normalize_embeddings: bool,
    pub batch_size: u32,
    pub workers: usize,
    pub queue_capacity: usize,
    pub model_name: String,
}

impl Config {
    pub fn from_env_or_file() -> Self {
        let mut config = Self::from_file().unwrap_or_else(Self::from_defaults);
        config.apply_env_overrides();
        config
    }

    fn from_file() -> Option<Self> {
        let path = Path::new("config.toml");
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        toml::from_str::<Config>(&content).ok()
    }

    fn from_defaults() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);
        let backend_url = env::var("BACKEND_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        let backend_type = env::var("BACKEND_TYPE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(BackendType::Proxy);
        let model_path = env::var("MODEL_PATH")
            .unwrap_or_else(|_| "/Users/zhangming/Documents/workspace/ZS/fj/yuan_model/Yuan-embedding-2.0-zh".to_string());
        let normalize_embeddings = env::var("NORMALIZE_EMBEDDINGS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true);
        let batch_size = env::var("BATCH_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(32);
        let workers = env::var("WORKERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let queue_capacity = env::var("QUEUE_CAPACITY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        let model_name = env::var("MODEL_NAME").unwrap_or_else(|_| "yuan-embedding-2.0-zh".to_string());

        Self {
            host,
            port,
            backend_url,
            backend_type,
            model_path,
            normalize_embeddings,
            batch_size,
            workers,
            queue_capacity,
            model_name,
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(host) = env::var("HOST") {
            self.host = host;
        }
        if let Ok(port) = env::var("PORT") {
            if let Ok(value) = port.parse() {
                self.port = value;
            }
        }
        if let Ok(backend_url) = env::var("BACKEND_URL") {
            self.backend_url = backend_url;
        }
        if let Ok(value) = env::var("BACKEND_TYPE") {
            if let Ok(v) = value.parse() {
                self.backend_type = v;
            }
        }
        if let Ok(value) = env::var("MODEL_PATH") {
            self.model_path = value;
        }
        if let Ok(value) = env::var("NORMALIZE_EMBEDDINGS") {
            if let Ok(v) = value.parse() {
                self.normalize_embeddings = v;
            }
        }
        if let Ok(value) = env::var("BATCH_SIZE") {
            if let Ok(v) = value.parse() {
                self.batch_size = v;
            }
        }
        if let Ok(value) = env::var("WORKERS") {
            if let Ok(v) = value.parse() {
                self.workers = v;
            }
        }
        if let Ok(value) = env::var("QUEUE_CAPACITY") {
            if let Ok(v) = value.parse() {
                self.queue_capacity = v;
            }
        }
        if let Ok(value) = env::var("MODEL_NAME") {
            self.model_name = value;
        }
    }
}
