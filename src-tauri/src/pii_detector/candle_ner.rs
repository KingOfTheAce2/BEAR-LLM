use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType, safetensors};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use tracing::info;
use crate::pii_detector::PIIEntity;

pub struct NerModel {
    model: BertModel,
    tokenizer: Tokenizer,
    id_to_label: Vec<String>,
    device: Device,
}

impl NerModel {
    pub fn new(model_id: &str, revision: &str, device: Device) -> Result<Self> {
        info!("Initializing Candle NER model: {} (revision: {}) on device: {:?}", model_id, revision, device);
        let api = Api::new()?;
        let repo = api.repo(Repo::with_revision(model_id.to_string(), RepoType::Model, revision.to_string()));

        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let model_filename = repo.get("model.safetensors")?; // Or "pytorch_model.bin"

        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| anyhow!("Failed to load tokenizer: {:?}", e))?;

        let config: BertConfig = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;

        let model_weights = safetensors::load(&model_filename, &device)?;
        let vb = VarBuilder::from_tensors(model_weights, DType::F32, &device)?;
        let model = BertModel::load(vb, &config)?;

        // This mapping needs to be accurate for the chosen model.
        // For 'dbmdz/bert-large-cased-finetuned-conll03-english', common labels are:
        let id_to_label = vec![
            "O".to_string(),
            "B-PER".to_string(), "I-PER".to_string(),
            "B-ORG".to_string(), "I-ORG".to_string(),
            "B-LOC".to_string(), "I-LOC".to_string(),
            "B-MISC".to_string(), "I-MISC".to_string(),
        ];

        info!("Candle NER model initialized successfully.");
        Ok(Self {
            model,
            tokenizer,
            id_to_label,
            device,
        })
    }

    pub fn predict(&mut self, text: &str) -> Result<Vec<PIIEntity>> {
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| anyhow!("Failed to encode text: {:?}", e))?;

        let tokens = encoding.get_ids().to_vec();
        let offsets = encoding.get_offsets().to_vec();

        let token_ids = Tensor::new(tokens.as_slice(), &self.device)?.unsqueeze(0)?; // Add batch dimension
        let attention_mask_vec = vec![1u32; tokens.len()];
        let attention_mask = Tensor::new(attention_mask_vec.as_slice(), &self.device)?.unsqueeze(0)?;

        let ys = self.model.forward(&token_ids, &attention_mask, None)?; // Get logits
        let logits = ys.squeeze(0)?.to_vec2::<f32>()?; // Remove batch dim and convert to Vec<Vec<f32>>

        let mut entities = Vec::new();
        let mut current_entity_tokens = Vec::new();
        let mut current_entity_label: Option<String> = None;
        let mut current_entity_start: Option<usize> = None;
        let mut current_entity_end: Option<usize> = None;
        let mut current_entity_confidence: f32 = 0.0;

        for (i, logit_row) in logits.iter().enumerate() {
            let (label_id, max_logit) = logit_row.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();

            let label = &self.id_to_label[label_id];
            let (token_start, token_end) = offsets[i];
            let token_text = &text[token_start..token_end];

            // Simple confidence: softmax of max logit (approximation)
            let confidence = 1.0 / (1.0 + (-max_logit).exp());

            if label.starts_with("B-") {
                if let Some(prev_label) = current_entity_label.take() {
                    if !current_entity_tokens.is_empty() {
                        let entity_text = current_entity_tokens.join("").replace("##", "");
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
                current_entity_tokens = vec![token_text.to_string()];
                current_entity_label = Some(label[2..].to_string());
                current_entity_start = Some(token_start);
                current_entity_end = Some(token_end);
                current_entity_confidence = confidence;
            } else if label.starts_with("I-") && current_entity_label.as_deref() == Some(&label[2..]) {
                current_entity_tokens.push(token_text.to_string());
                current_entity_end = Some(token_end);
                current_entity_confidence = (current_entity_confidence + confidence) / 2.0; // Average confidence
            } else {
                if let Some(prev_label) = current_entity_label.take() {
                    if !current_entity_tokens.is_empty() {
                        let entity_text = current_entity_tokens.join("").replace("##", "");
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

        if let Some(prev_label) = current_entity_label.take() {
            if !current_entity_tokens.is_empty() {
                let entity_text = current_entity_tokens.join("").replace("##", "");
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
