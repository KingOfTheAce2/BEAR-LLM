// Commands Module - Tauri Command Exports
// Organizes all Tauri commands for frontend integration

// FIXME: Consent commands disabled - requires middleware module
// pub mod consent_commands;
pub mod model_transparency;
pub mod scheduler_commands;
pub mod transparency_commands;

// FIXME: Consent command exports disabled
// pub use consent_commands::{
//     check_consent_status,
//     grant_consent,
//     revoke_consent,
//     check_multiple_consents,
//     get_consent_history,
//     check_reconsent_needed,
//     grant_all_consents,
//     revoke_all_consents,
//     get_consent_statistics,
// };

// Transparency commands exported directly from transparency_commands module

// Scheduler commands exported directly from scheduler_commands module
// Note: These commands are not currently registered in main.rs
#[allow(unused_imports)]
pub use scheduler_commands::{
    apply_default_retention_policies, get_last_cleanup_result, get_scheduler_status,
    preview_retention_cleanup, set_automatic_cleanup, trigger_retention_cleanup,
    update_scheduler_config,
};

// Model transparency commands exported directly from model_transparency module
// Note: These commands are not currently registered in main.rs
#[allow(unused_imports)]
pub use model_transparency::{
    add_model_mapping,
    clear_all_model_cache,
    clear_model_cache,
    // format_disclaimer_display, format_generic_disclaimer_display, get_high_risk_disclaimer, // Unused
    get_ai_act_disclaimer,
    get_general_disclaimer,
    get_model_info,
    get_model_mappings,
    remove_model_mapping,
    ModelTransparencyState,
};
