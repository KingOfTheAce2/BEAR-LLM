// Commands Module - Tauri Command Exports
// Organizes all Tauri commands for frontend integration

pub mod consent_commands;
pub mod model_transparency;
pub mod scheduler_commands;
pub mod transparency_commands;

// Consent commands exported directly from consent_commands module
pub use consent_commands::{
    check_consent_status,
    check_multiple_consents,
    check_reconsent_needed,
    get_consent_history,
    grant_all_consents,
    grant_consent,
    revoke_all_consents,
    revoke_consent,
};

// Transparency commands exported directly from transparency_commands module

// Scheduler commands exported directly from scheduler_commands module
pub use scheduler_commands::{
    get_last_cleanup_result,
    get_scheduler_status,
    preview_retention_cleanup,
    set_automatic_cleanup,
    trigger_retention_cleanup,
    update_scheduler_config,
};

// Model transparency commands exported directly from model_transparency module
pub use model_transparency::{
    add_model_mapping, clear_all_model_cache, clear_model_cache, get_ai_act_disclaimer,
    get_general_disclaimer, get_model_info, get_model_mappings, remove_model_mapping,
    ModelTransparencyState,
};
