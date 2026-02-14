use async_trait::async_trait;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use std::sync::Arc;
use tokenizers::{Tokenizer, TruncationDirection};

use crate::backend::{BackendError, EmbeddingBackend};
use crate::types::EmbedResponse;

#[derive(Clone)]
pub struct CandleBackend {
    model: Arc<BertModel>,
    tokenizer: Arc<Tokenizer>,
    device: Device,
    model_path: String,
}

impl CandleBackend {
    pub fn new(model_path: String) -> Result<Self, BackendError> {
        let device = Device::Cpu;
        
        // 加载 tokenizer
        let tokenizer = Tokenizer::from_file(format!("{}/tokenizer.json", model_path))
            .map_err(|e| BackendError::Request(format!("Failed to load tokenizer: {}", e)))?;
        
        // 加载模型配置
        let config = Config {
            vocab_size: 21128,
            hidden_size: 1024,
            num_hidden_layers: 24,
            num_attention_heads: 16,
            intermediate_size: 4096,
            max_position_embeddings: 512,
            type_vocab_size: 2,
            ..Config::default()
        };
        
        // 加载模型权重
        use candle_core::safetensors::load;
        let model_file = format!("{}/model.safetensors", model_path);
        let weights = load(model_file.as_str(), &device)
            .map_err(|e| BackendError::Request(format!("Failed to load weights: {}", e)))?;
        let vb = VarBuilder::from_tensors(weights, candle_core::DType::F32, &device);
        
        let model = BertModel::load(vb, &config)
            .map_err(|e| BackendError::Request(format!("Failed to load model: {}", e)))?;
        
        Ok(Self {
            model: Arc::new(model),
            tokenizer: Arc::new(tokenizer),
            device,
            model_path,
        })
    }
    
    fn encode_texts(&self, texts: &[String], batch_size: u32) -> Result<Vec<Vec<f32>>, BackendError> {
        let batch_size = batch_size as usize;
        let mut all_embeddings = Vec::new();
        
        // 批量处理
        for batch in texts.chunks(batch_size) {
            let batch_embeddings = self.process_batch(batch)?;
            all_embeddings.extend(batch_embeddings);
        }
        
        Ok(all_embeddings)
    }
    
    fn process_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, BackendError> {
        let tokenizer = self.tokenizer.clone();
        let model = self.model.clone();
        let device = &self.device;
        
        // 对文本进行 tokenize
        let tokenized = texts.iter().map(|text| {
            let text_str = text.as_str();
            let mut encoding = tokenizer.encode(text_str, true).unwrap();
            encoding.truncate(512, 0, TruncationDirection::Right); // 限制长度
            encoding
        }).collect::<Vec<_>>();
        
        // 准备输入张量
        let max_len = tokenized.iter().map(|e| e.len()).max().unwrap_or(1);
        let mut input_ids = Vec::new();
        let mut attention_mask = Vec::new();
        for encoding in &tokenized {
            let mut ids = encoding.get_ids().to_vec();
            let mut mask = encoding.get_attention_mask().to_vec();
            // 填充到最大长度
            while ids.len() < max_len {
                ids.push(0); // [PAD] token
                mask.push(0); // 注意力掩码
            }
            input_ids.push(ids);
            attention_mask.push(mask);
        }
        
        // 转换为i64类型
        let input_ids_i64: Vec<Vec<i64>> = input_ids.iter().map(|ids| ids.iter().map(|&id| id as i64).collect()).collect();
        let attention_mask_i64: Vec<Vec<i64>> = attention_mask.iter().map(|mask| mask.iter().map(|&m| m as i64).collect()).collect();
        
        // 创建输入张量
        let batch_size = input_ids_i64.len();
        let input_ids_flat: Vec<i64> = input_ids_i64.into_iter().flatten().collect();
        let attention_mask_flat: Vec<i64> = attention_mask_i64.into_iter().flatten().collect();
        
        let input_ids = Tensor::from_vec(input_ids_flat, (batch_size, max_len), device)
            .map_err(|e| BackendError::Request(format!("Failed to create input tensor: {}", e)))?;
        let attention_mask = Tensor::from_vec(attention_mask_flat, (batch_size, max_len), device)
            .map_err(|e| BackendError::Request(format!("Failed to create attention mask tensor: {}", e)))?;
        
        // 前向推理
        let output = model.forward(&input_ids, &attention_mask, None)
            .map_err(|e| BackendError::Request(format!("Model forward failed: {}", e)))?;
        
        // 获取 [CLS] 标记的输出作为 embedding
        // 使用index_select方法获取第一个 token (CLS) 的输出
        let cls_index = Tensor::from_vec(vec![0i64], (1,), device)
            .map_err(|e| BackendError::Request(format!("Failed to create CLS index: {}", e)))?;
        let embeddings = output
            .index_select(&cls_index, 1)
            .map_err(|e| BackendError::Request(format!("Failed to select CLS token: {}", e)))?
            .squeeze(1)
            .map_err(|e| BackendError::Request(format!("Failed to squeeze tensor: {}", e)))?
            .to_vec2()
            .map_err(|e| BackendError::Request(format!("Failed to extract embeddings: {}", e)))?;
        
        Ok(embeddings)
    }
    
    fn normalize(&self, embeddings: &mut Vec<Vec<f32>>) {
        for embedding in embeddings {
            let norm = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
            if norm > 1e-6 {
                for x in embedding {
                    *x /= norm;
                }
            }
        }
    }
}

#[async_trait]
impl EmbeddingBackend for CandleBackend {
    async fn embed(
        &self,
        texts: Vec<String>,
        normalize_embeddings: bool,
        batch_size: u32,
    ) -> Result<EmbedResponse, BackendError> {
        let mut embeddings = self.encode_texts(&texts, batch_size)?;
        
        // 归一化向量
        if normalize_embeddings {
            self.normalize(&mut embeddings);
        }
        
        let vector_dim = if embeddings.is_empty() {
            0
        } else {
            embeddings[0].len()
        };
        
        Ok(EmbedResponse {
            vectors: embeddings,
            count: texts.len(),
            vector_dim,
            model_path: self.model_path.clone(),
        })
    }
}
