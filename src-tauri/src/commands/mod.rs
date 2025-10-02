// Commands Module - Tauri Command Exports
// Organizes all Tauri commands for frontend integration

pub mod consent_commands;
pub mod transparency_commands;
pub mod scheduler_commands;
pub mod model_transparency;

pub use consent_commands::{
    check_consent_status,
    grant_consent,
    revoke_consent,
    check_multiple_consents,
    get_consent_history,
    check_reconsent_needed,
    grant_all_consents,
    revoke_all_consents,
    get_consent_statistics,
};

pub use transparency_commands::{
    get_startup_notice,
    get_onboarding_notice,
    get_limitations_notice,
    get_data_processing_notice,
    get_legal_disclaimer,
    create_transparency_context,
    get_transparency_notice,
    calculate_confidence_score,
    get_transparency_preferences,
    update_transparency_preferences,
    complete_onboarding,
    needs_disclaimer,
    acknowledge_disclaimers,
    get_all_notices,
    export_transparency_context,
};

pub use scheduler_commands::{
    trigger_retention_cleanup,
    get_scheduler_status,
    update_scheduler_config,
    preview_retention_cleanup,
    apply_default_retention_policies,
    get_last_cleanup_result,
    set_automatic_cleanup,
};

pub use model_transparency::{
    get_model_info,
    add_model_mapping,
    remove_model_mapping,
    get_model_mappings,
    clear_model_cache,
    clear_all_model_cache,
    get_general_disclaimer,
    get_ai_act_disclaimer,
    get_high_risk_disclaimer,
    format_disclaimer_display,
    format_generic_disclaimer_display,
    ModelTransparencyState,
};
