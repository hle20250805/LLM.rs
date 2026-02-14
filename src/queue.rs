use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::error;

use crate::backend::{BackendError, EmbeddingBackend};
use crate::types::EmbedResponse;

#[derive(Clone)]
pub struct Queue {
    sender: mpsc::Sender<EmbedJob>,
}

pub struct EmbedJob {
    pub texts: Vec<String>,
    pub normalize_embeddings: bool,
    pub batch_size: u32,
    pub response: oneshot::Sender<Result<EmbedResponse, BackendError>>,
}

impl Queue {
    pub fn new(
        backend: Arc<dyn EmbeddingBackend>,
        workers: usize,
        capacity: usize,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<EmbedJob>(capacity);
        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..workers.max(1) {
            let rx = receiver.clone();
            let backend = backend.clone();
            tokio::spawn(async move {
                loop {
                    let job = {
                        let mut guard = rx.lock().await;
                        guard.recv().await
                    };

                    let Some(job) = job else { break };

                    let result = backend
                        .embed(job.texts, job.normalize_embeddings, job.batch_size)
                        .await;
                    if job.response.send(result).is_err() {
                        error!("response channel dropped");
                    }
                }
            });
        }

        Self { sender }
    }

    pub async fn enqueue(
        &self,
        texts: Vec<String>,
        normalize_embeddings: bool,
        batch_size: u32,
    ) -> Result<EmbedResponse, BackendError> {
        let (tx, rx) = oneshot::channel();
        let job = EmbedJob {
            texts,
            normalize_embeddings,
            batch_size,
            response: tx,
        };

        self.sender
            .send(job)
            .await
            .map_err(|_| BackendError::Request("queue send failed".to_string()))?;

        rx.await
            .map_err(|_| BackendError::Request("queue response dropped".to_string()))?
    }
}
