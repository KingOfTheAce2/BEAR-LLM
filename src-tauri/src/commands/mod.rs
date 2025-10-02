// Commands Module - Tauri Command Exports
// Organizes all Tauri commands for frontend integration

// FIXME: Consent commands disabled - requires middleware module
// pub mod consent_commands;
pub mod transparency_commands;
pub mod scheduler_commands;
pub mod model_transparency;

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

// Model transparency commands exported directly from model_transparency module
pub use model_transparency::{
    get_high_risk_disclaimer,
    format_disclaimer_display,
    format_generic_disclaimer_display,
    ModelTransparencyState,
};
