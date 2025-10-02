/// Tauri Commands for AI Transparency Features
///
/// Exposes transparency functionality to the frontend application.

use crate::ai_transparency::{
    TransparencyContext, TransparencyPreferences, RiskLevel,
    confidence::{ConfidenceScore, ConfidenceFactors},
    notices::NoticeTemplates,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// State for transparency preferences
pub struct TransparencyState {
    preferences: Arc<RwLock<TransparencyPreferences>>,
    notices: NoticeTemplates,
}

impl Default for TransparencyState {
    fn default() -> Self {
        Self {
            preferences: Arc::new(RwLock::new(TransparencyPreferences::default())),
            notices: NoticeTemplates::default(),
        }
    }
}

impl TransparencyState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Request to create transparency context
#[derive(Debug, Deserialize)]
pub struct CreateTransparencyContextRequest {
    pub model_name: String,
    pub is_legal_advice: bool,
    pub affects_rights: bool,
    pub confidence: Option<f32>,
}

/// Response metadata for confidence scoring
#[derive(Debug, Deserialize)]
pub struct ResponseMetadata {
    pub token_count: usize,
    pub has_citations: bool,
    pub context_tokens: usize,
    pub coherence_score: Option<f32>,
}

/// Get startup disclaimer notice
#[tauri::command]
pub async fn get_startup_notice(
    state: tauri::State<'_, TransparencyState>,
) -> Result<String, String> {
    Ok(state.notices.startup.to_formatted_string())
}

/// Get onboarding notice for first-time users
#[tauri::command]
pub async fn get_onboarding_notice(
    state: tauri::State<'_, TransparencyState>,
) -> Result<String, String> {
    Ok(state.notices.onboarding.to_formatted_string())
}

/// Get model limitations notice
#[tauri::command]
pub async fn get_limitations_notice(
    state: tauri::State<'_, TransparencyState>,
) -> Result<String, String> {
    Ok(state.notices.limitations.to_formatted_string())
}

/// Get data processing transparency notice
#[tauri::command]
pub async fn get_data_processing_notice(
    state: tauri::State<'_, TransparencyState>,
) -> Result<String, String> {
    Ok(state.notices.data_processing.to_formatted_string())
}

/// Get legal disclaimer for specific context
#[tauri::command]
pub async fn get_legal_disclaimer(
    state: tauri::State<'_, TransparencyState>,
    is_high_risk: bool,
    includes_citations: bool,
) -> Result<String, String> {
    Ok(state.notices.legal_disclaimer.get_disclaimer(is_high_risk, includes_citations))
}

/// Create transparency context for an AI interaction
#[tauri::command]
pub async fn create_transparency_context(
    request: CreateTransparencyContextRequest,
) -> Result<TransparencyContext, String> {
    let risk_level = RiskLevel::from_context(
        request.is_legal_advice,
        request.affects_rights,
    );

    let mut context = TransparencyContext::new(request.model_name, risk_level);

    if let Some(confidence) = request.confidence {
        context = context.with_confidence(confidence);
    }

    Ok(context)
}

/// Get formatted transparency notice for a context
#[tauri::command]
pub async fn get_transparency_notice(
    context: TransparencyContext,
) -> Result<String, String> {
    Ok(context.get_notice())
}

/// Calculate confidence score for a response
#[tauri::command]
pub async fn calculate_confidence_score(
    metadata: ResponseMetadata,
) -> Result<ConfidenceScore, String> {
    let factors = ConfidenceFactors::from_response_metadata(
        metadata.token_count,
        metadata.has_citations,
        metadata.context_tokens,
        metadata.coherence_score,
    );

    Ok(ConfidenceScore::new(factors))
}

/// Get user transparency preferences
#[tauri::command]
pub async fn get_transparency_preferences(
    state: tauri::State<'_, TransparencyState>,
) -> Result<TransparencyPreferences, String> {
    let prefs = state.preferences.read().await;
    Ok(prefs.clone())
}

/// Update user transparency preferences
#[tauri::command]
pub async fn update_transparency_preferences(
    state: tauri::State<'_, TransparencyState>,
    preferences: TransparencyPreferences,
) -> Result<(), String> {
    let mut prefs = state.preferences.write().await;
    *prefs = preferences;
    Ok(())
}

/// Mark onboarding as completed
#[tauri::command]
pub async fn complete_onboarding(
    state: tauri::State<'_, TransparencyState>,
) -> Result<(), String> {
    let mut prefs = state.preferences.write().await;
    *prefs = prefs.clone().complete_onboarding();
    Ok(())
}

/// Check if disclaimer needs to be shown
#[tauri::command]
pub async fn needs_disclaimer(
    state: tauri::State<'_, TransparencyState>,
) -> Result<bool, String> {
    let prefs = state.preferences.read().await;
    Ok(prefs.needs_disclaimer())
}

/// Acknowledge disclaimers
#[tauri::command]
pub async fn acknowledge_disclaimers(
    state: tauri::State<'_, TransparencyState>,
) -> Result<(), String> {
    let mut prefs = state.preferences.write().await;
    *prefs = prefs.clone().complete_onboarding();
    Ok(())
}

/// Get all transparency notices as JSON
#[derive(Serialize)]
pub struct AllNotices {
    pub startup: String,
    pub onboarding: String,
    pub limitations: String,
    pub data_processing: String,
}

#[tauri::command]
pub async fn get_all_notices(
    state: tauri::State<'_, TransparencyState>,
) -> Result<AllNotices, String> {
    Ok(AllNotices {
        startup: state.notices.startup.to_formatted_string(),
        onboarding: state.notices.onboarding.to_formatted_string(),
        limitations: state.notices.limitations.to_formatted_string(),
        data_processing: state.notices.data_processing.to_formatted_string(),
    })
}

/// Export transparency context as JSON
#[tauri::command]
pub async fn export_transparency_context(
    context: TransparencyContext,
) -> Result<String, String> {
    serde_json::to_string_pretty(&context)
        .map_err(|e| format!("Failed to serialize context: {}", e))
}

#[cfg(test)]
mod command_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_transparency_context() {
        let request = CreateTransparencyContextRequest {
            model_name: "test-model".to_string(),
            is_legal_advice: true,
            affects_rights: true,
            confidence: Some(0.75),
        };

        let result = create_transparency_context(request).await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.risk_level, RiskLevel::High);
        assert_eq!(context.confidence, 0.75);
    }

    #[tokio::test]
    async fn test_calculate_confidence_score() {
        let metadata = ResponseMetadata {
            token_count: 150,
            has_citations: true,
            context_tokens: 800,
            coherence_score: Some(0.8),
        };

        let result = calculate_confidence_score(metadata).await;
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score.overall > 0.0);
        assert!(score.overall <= 1.0);
    }

    #[tokio::test]
    async fn test_transparency_state() {
        let state = TransparencyState::new();
        let prefs = state.preferences.read().await;

        assert!(prefs.show_startup_disclaimer);
        assert!(!prefs.onboarding_completed);
    }
}
