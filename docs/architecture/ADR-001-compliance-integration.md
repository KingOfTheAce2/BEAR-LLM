# ADR-001: Compliance Integration Architecture

**Status:** Proposed
**Date:** 2025-10-02
**Decision Makers:** System Architecture Team
**Affected Components:** Main Application State, Chat System, Model Loading, PII Detection, Setup Flow

---

## Context

BEAR-LLM requires comprehensive GDPR and AI Act compliance with three parallel features:
1. **Chat Encryption** - End-to-end encrypted message storage
2. **AI Transparency** - Model card fetching and disclaimer system
3. **PII Configuration** - Presidio integration with resource-aware fallback

These features must integrate seamlessly into the existing application architecture while maintaining:
- **Security First**: All chat data encrypted before database storage
- **Consent Enforcement**: Operations blocked without valid user consent
- **Transparency**: Users informed about AI limitations before first use
- **Resource Awareness**: PII detection mode adapts to system memory
- **User Experience**: Smooth onboarding without friction

---

## Decision

### 1. Unified Application State Architecture

We will extend `AppState` to include all compliance components as first-class citizens:

```rust
pub struct AppState {
    // Existing services
    pii_detector: Arc<RwLock<PIIDetector>>,
    rag_engine: Arc<RwLock<RAGEngine>>,
    llm_manager: Arc<RwLock<LLMManager>>,

    // NEW: Compliance & Security Services
    chat_encryption: Arc<RwLock<ChatEncryption>>,
    consent_guard: Arc<ConsentGuard>,
    compliance_manager: Arc<ComplianceManager>,

    // NEW: AI Transparency
    transparency_state: Arc<TransparencyState>,
    model_card_fetcher: Arc<RwLock<ModelCardFetcher>>,

    // NEW: PII Configuration
    pii_config_manager: Arc<RwLock<PIIConfigManager>>,

    // Existing...
    database_manager: Arc<RwLock<DatabaseManager>>,
    // ...
}
```

**Rationale:**
- Centralized access to all services via Tauri State management
- Type-safe access with compile-time guarantees
- Async-ready with RwLock for concurrent operations
- Clear ownership and lifecycle management

---

### 2. Chat Flow Integration Pattern

**BEFORE ANY DATABASE OPERATION:**

```
User Message Input
     ↓
[1] Check Consent (ConsentGuard)
     ↓ (denied) → Show Consent UI → Retry
     ↓ (granted)
[2] Check Model Disclaimer Acknowledgment
     ↓ (not seen) → Show Model Card → Wait for ACK
     ↓ (acknowledged)
[3] PII Detection & Redaction
     ↓
[4] Encrypt Message Content
     ↓
[5] Store Encrypted in Database
     ↓
[6] Log Audit Trail (success/failure)
```

**Implementation:**

```rust
async fn send_message(
    state: State<AppState>,
    user_id: String,
    message: String,
    model: String,
) -> Result<String, String> {
    // [1] Consent check
    state.consent_guard
        .enforce_consent(&user_id, &ConsentType::ChatStorage)
        .await?;

    // [2] Model disclaimer check
    if !has_seen_model_disclaimer(&state, &user_id, &model).await? {
        return Err(ComplianceError::DisclaimerRequired {
            model: model.clone(),
            model_card_url: fetch_model_card_url(&model),
        }.to_string());
    }

    // [3] PII Detection
    let pii_detector = state.pii_detector.read().await;
    let cleaned_message = pii_detector.redact_pii(&message).await?;
    drop(pii_detector);

    // [4] Encryption
    let encryption = state.chat_encryption.write().await;
    let encrypted = encryption.encrypt(&cleaned_message, &user_id).await?;
    drop(encryption);

    // [5] Database storage
    let db = state.database_manager.read().await;
    let message_id = db.save_encrypted_message(
        &user_id,
        &encrypted,
        &model,
        chrono::Utc::now(),
    )?;
    drop(db);

    // [6] Audit logging
    state.compliance_manager
        .audit()
        .write()
        .await
        .log_success(
            &user_id,
            AuditAction::DataModified,
            EntityType::ChatMessage,
            Some(&message_id),
            Some(serde_json::json!({"model": model, "encrypted": true})),
        )?;

    // Generate AI response...
    Ok(response)
}
```

---

### 3. Model Loading Integration Pattern

**ON MODEL LOAD:**

```
Model Selection
     ↓
[1] Fetch Model Card (async, non-blocking)
     ↓ (success)
[2] Display Model Info Panel
     ↓
[3] Show Limitations Disclaimer
     ↓
[4] Wait for User Acknowledgment
     ↓ (acknowledged)
[5] Store Acknowledgment in DB
     ↓
[6] Load Model Weights
     ↓
[7] Enable Chat Interface
```

**Database Schema:**

```sql
CREATE TABLE model_acknowledgments (
    user_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    acknowledged_at INTEGER NOT NULL,
    model_card_version TEXT,
    PRIMARY KEY (user_id, model_name)
);
```

**Benefits:**
- One-time acknowledgment per user per model
- Version tracking for model card updates
- Non-blocking UI (model loads while disclaimer shown)

---

### 4. PII Configuration Strategy

**Resource-Aware PII Mode Selection:**

```
Startup
     ↓
[1] Detect System Memory
     ↓
     ├─ < 4GB RAM → Built-in PII (0MB overhead)
     ├─ 4-8GB RAM → Presidio Lite (200-400MB)
     └─ > 8GB RAM → Full Presidio (800MB-1.2GB)
     ↓
[2] Check Presidio Installation
     ↓ (not installed)
     └─ Show Installation Guide + Fallback to Built-in
     ↓ (installed)
[3] Initialize Selected Mode
     ↓
[4] Update UI Status Indicator
```

**Configuration Storage:**

```rust
pub struct PIIConfig {
    mode: PIIMode,           // BuiltIn, PresidioLite, PresidioFull
    auto_select: bool,       // Enable resource-aware selection
    memory_limit_mb: u64,    // Max memory for PII detection
    fallback_enabled: bool,  // Fallback to built-in on error
}
```

---

### 5. Setup Wizard Architecture

**Multi-Step Onboarding Flow:**

```typescript
// Step 1: Welcome & GDPR Notice
interface WelcomeStep {
  title: "Welcome to BEAR AI";
  content: "GDPR compliance summary";
  required: true;
}

// Step 2: Consent Collection
interface ConsentStep {
  consents: [
    { type: "chat_storage", required: true },
    { type: "pii_detection", required: true },
    { type: "ai_processing", required: true },
    { type: "analytics", required: false },
    { type: "document_processing", required: false },
  ];
}

// Step 3: PII Configuration
interface PIIConfigStep {
  auto_detect: boolean;
  manual_mode?: "builtin" | "presidio-lite" | "presidio-full";
  ram_check: SystemMemoryInfo;
}

// Step 4: Chat Encryption Setup
interface EncryptionStep {
  method: "password" | "pin" | "system-keychain";
  password?: string;
  auto_unlock?: boolean;
}

// Step 5: Data Retention Preferences
interface RetentionStep {
  chat_retention_days: number;    // Default: 90
  document_retention_days: number; // Default: 365
  auto_cleanup: boolean;           // Default: true
}

// Step 6: Summary & Confirmation
interface SummaryStep {
  review: AllSettings;
  can_change_later: true;
  privacy_dashboard_link: "/settings/privacy";
}
```

**State Management:**

```typescript
interface SetupState {
  current_step: number;
  total_steps: 6;
  completed: boolean;
  settings: Partial<AllSettings>;
  can_skip?: boolean;
}
```

---

### 6. Settings Panel Architecture

**Tabbed Interface:**

```
┌─ Privacy & Security ────────────────────────────────┐
│                                                      │
│  🔒 Chat Encryption                                  │
│     Status: Enabled ✓                                │
│     Method: System Keychain                          │
│     [Change Password]                                │
│                                                      │
│  🛡️ PII Detection                                    │
│     Mode: Built-in (0MB)                             │
│     Status: Active ✓                                 │
│     [Configure] [Test Detection]                     │
│                                                      │
│  ⏰ Data Retention                                    │
│     Chats: 90 days                                   │
│     Documents: 365 days                              │
│     [Modify Retention]                               │
│                                                      │
└──────────────────────────────────────────────────────┘

┌─ AI Transparency ───────────────────────────────────┐
│                                                      │
│  📋 Active Model                                     │
│     Name: Llama-2-7B-Chat                            │
│     Model Card: [View]                               │
│     Risk Level: Limited                              │
│                                                      │
│  ⚠️ Disclaimers                                      │
│     ✓ Startup notice acknowledged                    │
│     ✓ Model limitations acknowledged                 │
│     [Review Disclaimers]                             │
│                                                      │
│  📊 Confidence Display                               │
│     Show confidence scores: [✓]                      │
│     Minimum threshold: 70%                           │
│                                                      │
└──────────────────────────────────────────────────────┘

┌─ Consent Management ────────────────────────────────┐
│                                                      │
│  ✓ Chat Storage (granted 2025-09-01)                │
│  ✓ PII Detection (granted 2025-09-01)               │
│  ✓ AI Processing (granted 2025-09-01)               │
│  ✗ Analytics (not granted)                          │
│                                                      │
│  [Privacy Dashboard] [Manage Consents]              │
│                                                      │
└──────────────────────────────────────────────────────┘

┌─ Data Export & Deletion ────────────────────────────┐
│                                                      │
│  📥 Export Your Data (GDPR Art. 20)                  │
│     [Export to JSON] [Export to CSV]                │
│                                                      │
│  🗑️ Delete Your Data (GDPR Art. 17)                 │
│     [Request Deletion] (irreversible)               │
│                                                      │
└──────────────────────────────────────────────────────┘
```

---

### 7. Error Handling Strategy

**Graceful Degradation Matrix:**

| Component | Failure Mode | Fallback Behavior | User Notification |
|-----------|-------------|-------------------|-------------------|
| Chat Encryption | Key unavailable | Block operation | "Unlock required" modal |
| Chat Encryption | Encryption failed | Block operation | Error + retry |
| PII Detection (Presidio) | Not installed | Use built-in | Warning banner |
| PII Detection (Presidio) | Runtime error | Use built-in | Warning + log |
| Model Card Fetcher | Network error | Show generic disclaimer | "Offline mode" notice |
| Model Card Fetcher | Invalid card | Show generic disclaimer | Warning |
| Consent Check | DB error | Block operation | Error modal |
| Consent Check | No consent | Show consent UI | Consent modal |

**Implementation Pattern:**

```rust
pub enum ComplianceError {
    ConsentRequired {
        consent_type: ConsentType,
        message: String,
    },
    DisclaimerRequired {
        model: String,
        model_card_url: Option<String>,
    },
    EncryptionFailed {
        reason: String,
    },
    PIIDetectionFailed {
        fallback_used: bool,
    },
}

impl ComplianceError {
    pub fn to_user_message(&self) -> String {
        match self {
            Self::ConsentRequired { consent_type, .. } => {
                format!("Consent required for {}. Please grant consent in Privacy Settings.", consent_type)
            }
            Self::DisclaimerRequired { model, .. } => {
                format!("Please review and acknowledge limitations for model: {}", model)
            }
            Self::EncryptionFailed { reason } => {
                format!("Cannot encrypt message: {}. Message not saved.", reason)
            }
            Self::PIIDetectionFailed { fallback_used: true } => {
                "PII detection degraded - using built-in detector.".to_string()
            }
            _ => "Operation failed due to compliance requirements.".to_string(),
        }
    }
}
```

---

### 8. Testing Strategy

**Test Levels:**

```
Unit Tests (per module)
     ↓
Integration Tests (component interactions)
     ↓
End-to-End Tests (full user flows)
     ↓
Compliance Tests (GDPR/AI Act requirements)
```

**E2E Test Scenarios:**

```rust
#[tokio::test]
async fn test_first_run_setup_flow() {
    // 1. User opens app for first time
    // 2. Setup wizard shown
    // 3. User grants consents
    // 4. User configures PII detection
    // 5. User sets up encryption password
    // 6. Setup marked complete
    // 7. App ready for use
}

#[tokio::test]
async fn test_chat_message_compliance_flow() {
    // 1. User loads model (sees disclaimer)
    // 2. User acknowledges disclaimer
    // 3. User sends message
    // 4. Message encrypted
    // 5. Message saved to DB (encrypted)
    // 6. Audit log created
    // 7. Message decrypted for display
}

#[tokio::test]
async fn test_consent_revocation_flow() {
    // 1. User revokes chat storage consent
    // 2. Attempt to send message
    // 3. Operation blocked
    // 4. Consent modal shown
    // 5. User re-grants consent
    // 6. Message sent successfully
}

#[tokio::test]
async fn test_pii_detection_fallback() {
    // 1. Presidio configured but unavailable
    // 2. System detects failure
    // 3. Falls back to built-in detector
    // 4. Warning shown to user
    // 5. Document still processed
}
```

---

## Consequences

### Positive

1. **Security by Default**: Chat encryption enforced before database storage
2. **GDPR Compliance**: Consent enforcement built into every data operation
3. **AI Act Compliance**: Transparency notices shown for all AI interactions
4. **Resource Efficiency**: PII detection adapts to system capabilities
5. **User Control**: Setup wizard ensures informed consent
6. **Maintainability**: Clear separation of concerns with modular architecture

### Negative

1. **Complexity**: More moving parts to maintain
2. **Performance Overhead**: Encryption adds latency to chat operations
3. **User Friction**: Consent and disclaimer flows may slow onboarding
4. **Database Migration**: Need to encrypt existing messages

### Mitigation Strategies

1. **Complexity**: Comprehensive documentation and ADRs
2. **Performance**: Async operations + caching + connection pooling
3. **User Friction**: Skip disclaimers for acknowledged models
4. **Migration**: Background migration task with progress indicator

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Extend AppState with new services
- [ ] Create chat encryption module
- [ ] Integrate consent guard into chat flow
- [ ] Unit tests

### Phase 2: AI Transparency (Week 2)
- [ ] Model card fetcher implementation
- [ ] Disclaimer acknowledgment system
- [ ] Transparency notices UI
- [ ] Integration tests

### Phase 3: PII Configuration (Week 3)
- [ ] Resource detection system
- [ ] PII mode selection logic
- [ ] Fallback mechanisms
- [ ] Configuration UI

### Phase 4: Setup Wizard (Week 4)
- [ ] Multi-step wizard component
- [ ] State persistence
- [ ] Settings panel integration
- [ ] E2E tests

### Phase 5: Testing & Documentation (Week 5)
- [ ] Compliance test suite
- [ ] Performance benchmarks
- [ ] User documentation
- [ ] Developer documentation

---

## References

- GDPR Article 32: Security of Processing
- GDPR Article 13: Information to be provided
- EU AI Act Article 13: Transparency obligations
- BEAR-LLM Architecture Documentation
- SQLCipher Documentation
- Microsoft Presidio Documentation

---

## Approval

- [ ] Security Team Review
- [ ] Compliance Team Review
- [ ] Engineering Team Review
- [ ] Product Team Review

**Decision Date:** TBD
**Review Date:** TBD (6 months after implementation)
