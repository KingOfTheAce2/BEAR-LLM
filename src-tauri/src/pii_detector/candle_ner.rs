use anyhow::{anyhow, Result};
use candle_core::{Device, Tensor, DType, safetensors};
use candle_nn::{linear, Linear, VarBuilder, Module};
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use tokenizers::Tokenizer;
use tracing::info;
use std::path::Path;
use std::fs;
use crate::pii_detector::PIIEntity;

/// Lightweight wrapper around BertModel for token classification (NER)
pub struct BertForTokenClassification {
    bert: BertModel,
    classifier: Linear,
}

impl BertForTokenClassification {
    pub fn load(vb: VarBuilder, config: &BertConfig, num_labels: usize) -> Result<Self> {
        // Load pretrained BERT encoder
        let bert = BertModel::load(vb.pp("bert"), config)
            .map_err(|e| anyhow!("Failed to load base BertModel: {}", e))?;

        // Create linear classification head
        let classifier = linear(config.hidden_size, num_labels, vb.pp("classifier"))
            .map_err(|e| anyhow!("Failed to create classifier layer: {}", e))?;

        Ok(Self { bert, classifier })
    }

    pub fn forward(&self, input_ids: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        // BERT forward signature in Candle 0.8.4:
        // forward(input_ids, attention_mask: Option<&Tensor>, token_type_ids: Option<&Tensor>)
        let hidden_states = self
            .bert
            .forward(input_ids, attention_mask, None)
            .map_err(|e| anyhow!("Forward pass failed: {}", e))?;

        let logits = self
            .classifier
            .forward(&hidden_states)
            .map_err(|e| anyhow!("Classifier forward failed: {}", e))?;

        Ok(logits)
    }
}

pub struct NerModel {
    model: BertForTokenClassification,
    tokenizer: Tokenizer,
    id_to_label: Vec<String>,
    device: Device,
}

impl NerModel {
    /// Create a new NER model using a local model directory (no API calls)
    pub fn new_local<P: AsRef<Path>>(model_dir: P, device: Device) -> Result<Self> {
        let model_dir = model_dir.as_ref().to_path_buf();
        info!("🧠 Initializing Candle NER model from local path: {:?}", model_dir);

        // Check required files
        let config_path = model_dir.join("config.json");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let safetensors_path = model_dir.join("model.safetensors");
        let pytorch_path = model_dir.join("pytorch_model.bin");

        if !config_path.exists() || !tokenizer_path.exists() {
            return Err(anyhow!(
                "Missing model files in {:?}. Required: config.json and tokenizer.json",
                model_dir
            ));
        }

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {:?}", e))?;

        // Load config
        let config_json = fs::read_to_string(&config_path)?;
        let config: BertConfig = serde_json::from_str(&config_json)
            .map_err(|e| anyhow!("Failed to parse config.json: {}", e))?;

        // Default NER label mapping (BIO format)
        let id_to_label = vec![
            "O".to_string(),
            "B-PER".to_string(),
            "I-PER".to_string(),
            "B-ORG".to_string(),
            "I-ORG".to_string(),
            "B-LOC".to_string(),
            "I-LOC".to_string(),
            "B-MISC".to_string(),
            "I-MISC".to_string(),
        ];
        let num_labels = id_to_label.len();

        // Load model weights (prefer safetensors)
        let model_path = if safetensors_path.exists() {
            safetensors_path
        } else if pytorch_path.exists() {
            pytorch_path
        } else {
            return Err(anyhow!(
                "No model weights found in {:?} (expected model.safetensors or pytorch_model.bin)",
                model_dir
            ));
        };

        // Load tensors into VarBuilder
        let model_weights = safetensors::load(&model_path, &device)
            .map_err(|e| anyhow!("Failed to load model weights: {}", e))?;
        let vb = VarBuilder::from_tensors(model_weights, DType::F32, &device);

        // Initialize model
        let model = BertForTokenClassification::load(vb, &config, num_labels)
            .map_err(|e| anyhow!("Failed to initialize Bert model: {}", e))?;

        info!(
            "✅ Candle NER model ready from local path {:?} ({} labels)",
            model_dir,
            id_to_label.len()
        );

        Ok(Self {
            model,
            tokenizer,
            id_to_label,
            device,
        })
    }

    /// Run prediction on input text and return detected entities
    pub fn predict(&mut self, text: &str) -> Result<Vec<PIIEntity>> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("Failed to encode text: {:?}", e))?;

        let tokens = encoding.get_ids().to_vec();
        let offsets = encoding.get_offsets().to_vec();

        let token_ids = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?; // [1, seq_len]
        let attention_mask_vec = vec![1u32; tokens.len()];
        let attention_mask =
            Tensor::new(attention_mask_vec.as_slice(), &self.device)?.unsqueeze(0)?; // [1, seq_len]

        let logits = self.model.forward(&token_ids, &attention_mask)?;
        let logits = logits.squeeze(0)?.to_vec2::<f32>()?; // [seq_len, num_labels]

        let mut entities = Vec::new();
        let mut current_entity_tokens = Vec::new();
        let mut current_entity_label: Option<String> = None;
        let mut current_entity_start: Option<usize> = None;
        let mut current_entity_end: Option<usize> = None;
        let mut current_entity_confidence: f32 = 0.0;

        for (i, logit_row) in logits.iter().enumerate() {
            let (start, end) = offsets[i];
            if start == 0 && end == 0 {
                continue; // skip special tokens
            }

            // Argmax over labels
            let (label_id, max_logit) = logit_row
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();

            let label = self
                .id_to_label
                .get(label_id)
                .cloned()
                .unwrap_or_else(|| "O".to_string())
                .to_uppercase();

            let token_text = &text[start..end];
            let sum_exp: f32 = logit_row.iter().map(|x| x.exp()).sum();
            let confidence = (max_logit.exp()) / sum_exp;

            if let Some(stripped) = label.strip_prefix("B-") {
                // End previous entity if one is open
                if let Some(prev_label) = current_entity_label.take() {
                    if !current_entity_tokens.is_empty() {
                        let entity_text = current_entity_tokens.join(" ").replace(" ##", "");
                        entities.push(PIIEntity {
                            entity_type: prev_label,
                            text: entity_text,
                            start: current_entity_start.unwrap_or(0),
                            end: current_entity_end.unwrap_or(0),
                            confidence: current_entity_confidence,
                            engine: "candle".to_string(),
                        });
                    }
                }

                // Start new entity
                current_entity_tokens = vec![token_text.to_string()];
                current_entity_label = Some(stripped.to_string());
                current_entity_start = Some(start);
                current_entity_end = Some(end);
                current_entity_confidence = confidence;
            } else if let Some(stripped) = label.strip_prefix("I-")
                && current_entity_label.as_deref() == Some(stripped)
            {
                // Continue current entity
                current_entity_tokens.push(token_text.to_string());
                current_entity_end = Some(end);
                current_entity_confidence =
                    (current_entity_confidence + confidence) / 2.0;
            } else {
                // Close current entity
                if let Some(prev_label) = current_entity_label.take() {
                    if !current_entity_tokens.is_empty() {
                        let entity_text = current_entity_tokens.join(" ").replace(" ##", "");
                        entities.push(PIIEntity {
                            entity_type: prev_label,
                            text: entity_text,
                            start: current_entity_start.unwrap_or(0),
                            end: current_entity_end.unwrap_or(0),
                            confidence: current_entity_confidence,
                            engine: "candle".to_string(),
                        });
                    }
                }
                current_entity_tokens.clear();
                current_entity_start = None;
                current_entity_end = None;
                current_entity_confidence = 0.0;
            }
        }

        // Push final entity if still open
        if let Some(prev_label) = current_entity_label.take() {
            if !current_entity_tokens.is_empty() {
                let entity_text = current_entity_tokens.join(" ").replace(" ##", "");
                entities.push(PIIEntity {
                    entity_type: prev_label,
                    text: entity_text,
                    start: current_entity_start.unwrap_or(0),
                    end: current_entity_end.unwrap_or(0),
                    confidence: current_entity_confidence,
                    engine: "candle".to_string(),
                });
            }
        }

        Ok(entities)
    }
}