use crate::ai_transparency::{
    DisclaimerGenerator, GenericDisclaimer, GenericDisclaimerGenerator, ModelCardFetcher,
    ModelCardParser, ModelDisclaimer, ModelRegistry,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct ModelTransparencyState {
    fetcher: Mutex<ModelCardFetcher>,
    registry: Mutex<ModelRegistry>,
    #[allow(dead_code)]
    cache_dir: PathBuf,
    config_path: PathBuf,
}

impl ModelTransparencyState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let cache_dir = app_data_dir.join("model_cards");
        let config_path = app_data_dir.join("model_mappings.json");

        let fetcher = ModelCardFetcher::new(cache_dir.clone());
        let registry = ModelRegistry::load_from_file(config_path.clone())
            .unwrap_or_else(|_| ModelRegistry::new());

        Self {
            fetcher: Mutex::new(fetcher),
            registry: Mutex::new(registry),
            cache_dir,
            config_path,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub filename: String,
    pub display_name: String,
    pub model_id: Option<String>,
    pub disclaimer: Option<ModelDisclaimer>,
    pub generic_disclaimer: Option<GenericDisclaimer>,
}

/// Get model information and disclaimer for a GGUF file
#[tauri::command]
pub async fn get_model_info(
    filename: String,
    state: State<'_, ModelTransparencyState>,
) -> Result<ModelInfo, String> {
    let display_name = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        registry.extract_model_name(&filename)
    };

    // Try to resolve model ID
    let model_id = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        registry.resolve_model_id(&filename)
    };

    if let Some(ref model_id) = model_id {
        // Try to fetch model card
        let fetcher = state.fetcher.lock().map_err(|e| e.to_string())?;
        match fetcher.fetch_model_card(model_id).await {
            Ok(cached_card) => {
                let model_card =
                    ModelCardParser::parse(model_id.clone(), &cached_card.readme_content);
                let disclaimer = DisclaimerGenerator::generate(&model_card);

                return Ok(ModelInfo {
                    filename,
                    display_name,
                    model_id: Some(model_id.clone()),
                    disclaimer: Some(disclaimer),
                    generic_disclaimer: None,
                });
            }
            Err(_) => {
                // Fallback to offline disclaimer
                let generic_disclaimer =
                    GenericDisclaimerGenerator::generate_offline_disclaimer(&display_name);

                return Ok(ModelInfo {
                    filename,
                    display_name: display_name.clone(),
                    model_id: Some(model_id.clone()),
                    disclaimer: None,
                    generic_disclaimer: Some(generic_disclaimer),
                });
            }
        }
    }

    // No model ID found - use generic unknown model disclaimer
    let generic_disclaimer = GenericDisclaimerGenerator::generate_unknown_model(&display_name);

    Ok(ModelInfo {
        filename,
        display_name,
        model_id: None,
        disclaimer: None,
        generic_disclaimer: Some(generic_disclaimer),
    })
}

/// Add custom model mapping
#[tauri::command]
pub async fn add_model_mapping(
    filename: String,
    model_id: String,
    state: State<'_, ModelTransparencyState>,
) -> Result<(), String> {
    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    registry.add_custom_mapping(filename, model_id);
    registry.save_to_file(state.config_path.clone())
}

/// Remove custom model mapping
#[tauri::command]
pub async fn remove_model_mapping(
    filename: String,
    state: State<'_, ModelTransparencyState>,
) -> Result<bool, String> {
    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    let removed = registry.remove_custom_mapping(&filename);
    if removed {
        registry.save_to_file(state.config_path.clone())?;
    }
    Ok(removed)
}

/// Get all model mappings
#[tauri::command]
pub async fn get_model_mappings(
    state: State<'_, ModelTransparencyState>,
) -> Result<Vec<(String, String)>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    Ok(registry.get_all_mappings())
}

/// Clear cache for a specific model
#[tauri::command]
pub async fn clear_model_cache(
    model_id: String,
    state: State<'_, ModelTransparencyState>,
) -> Result<(), String> {
    let fetcher = state.fetcher.lock().map_err(|e| e.to_string())?;
    fetcher.clear_cache(&model_id)
}

/// Clear all cached model cards
#[tauri::command]
pub async fn clear_all_model_cache(state: State<'_, ModelTransparencyState>) -> Result<(), String> {
    let fetcher = state.fetcher.lock().map_err(|e| e.to_string())?;
    fetcher.clear_all_cache()
}

/// Get general AI disclaimer
#[tauri::command]
pub async fn get_general_disclaimer() -> Result<GenericDisclaimer, String> {
    Ok(GenericDisclaimerGenerator::generate_general_ai_disclaimer())
}

/// Get EU AI Act disclaimer
#[tauri::command]
pub async fn get_ai_act_disclaimer() -> Result<GenericDisclaimer, String> {
    Ok(GenericDisclaimerGenerator::generate_ai_act_disclaimer())
}

/// Get high-risk application disclaimer
#[tauri::command]
pub async fn get_high_risk_disclaimer() -> Result<GenericDisclaimer, String> {
    Ok(GenericDisclaimerGenerator::generate_high_risk_disclaimer())
}

/// Format disclaimer for display
#[tauri::command]
pub async fn format_disclaimer_display(disclaimer: ModelDisclaimer) -> Result<String, String> {
    Ok(DisclaimerGenerator::format_for_display(&disclaimer))
}

/// Format generic disclaimer for display
#[tauri::command]
pub async fn format_generic_disclaimer_display(
    disclaimer: GenericDisclaimer,
) -> Result<String, String> {
    Ok(GenericDisclaimerGenerator::format_for_display(&disclaimer))
}
