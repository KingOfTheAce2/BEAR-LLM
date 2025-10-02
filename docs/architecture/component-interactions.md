# Component Interaction Diagrams - Compliance Integration

## Overview

This document describes how compliance components interact within the BEAR-LLM system.

---

## 1. System Component Architecture (C4 Level 2)

```
┌──────────────────────────────────────────────────────────────────────┐
│                         BEAR AI Application                          │
│                                                                      │
│  ┌────────────────────┐          ┌────────────────────┐            │
│  │   Frontend (React) │          │  Tauri Backend     │            │
│  │                    │   IPC    │   (Rust)           │            │
│  │  - Chat UI         │◄────────►│                    │            │
│  │  - Setup Wizard    │          │  ┌──────────────┐  │            │
│  │  - Settings Panel  │          │  │  AppState    │  │            │
│  │  - Privacy Dash    │          │  │  (Central)   │  │            │
│  └────────────────────┘          │  └──────┬───────┘  │            │
│                                  │         │          │            │
│                                  │         │          │            │
│  ┌─────────────────────────────────────────┴──────────────────┐   │
│  │                    Core Services Layer                      │   │
│  │                                                              │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐       │   │
│  │  │   Consent   │  │    Chat     │  │     PII      │       │   │
│  │  │    Guard    │  │ Encryption  │  │  Detection   │       │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬───────┘       │   │
│  │         │                │                 │               │   │
│  │  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼───────┐       │   │
│  │  │ Compliance  │  │  Security   │  │ Transparency │       │   │
│  │  │  Manager    │  │   Module    │  │    State     │       │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬───────┘       │   │
│  │         │                │                 │               │   │
│  └─────────┼────────────────┼─────────────────┼───────────────┘   │
│            │                │                 │                   │
│  ┌─────────▼────────────────▼─────────────────▼───────────────┐   │
│  │                 Database Layer (SQLite)                     │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │   │
│  │  │   Consents   │  │   Encrypted  │  │    Audit     │     │   │
│  │  │     Table    │  │   Messages   │  │     Logs     │     │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │   │
│  │                                                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 2. Chat Message Flow (Sequence Diagram)

```
User      ChatUI    Tauri      Consent    Chat       PII        Database    Audit
 │          │        │         Guard    Encryption  Detector      │          │
 │          │        │           │          │          │          │          │
 │  Type    │        │           │          │          │          │          │
 │ message  │        │           │          │          │          │          │
 │──────────►        │           │          │          │          │          │
 │          │        │           │          │          │          │          │
 │          │ send_  │           │          │          │          │          │
 │          │message │           │          │          │          │          │
 │          │────────►           │          │          │          │          │
 │          │        │           │          │          │          │          │
 │          │        │ check_    │          │          │          │          │
 │          │        │ consent   │          │          │          │          │
 │          │        │───────────►          │          │          │          │
 │          │        │           │          │          │          │          │
 │          │        │  ✓ allowed│          │          │          │          │
 │          │        │◄───────────          │          │          │          │
 │          │        │           │          │          │          │          │
 │          │        │ redact_pii│          │          │          │          │
 │          │        │────────────┼──────────┼─────────►          │          │
 │          │        │           │          │          │          │          │
 │          │        │      cleaned_text    │          │          │          │
 │          │        │◄─────────────────────┼──────────          │          │
 │          │        │           │          │          │          │          │
 │          │        │ encrypt   │          │          │          │          │
 │          │        │────────────┼─────────►          │          │          │
 │          │        │           │          │          │          │          │
 │          │        │   encrypted_data     │          │          │          │
 │          │        │◄─────────────────────          │          │          │
 │          │        │           │          │          │          │          │
 │          │        │ save_encrypted_message          │          │          │
 │          │        │──────────────────────┼──────────┼─────────►          │
 │          │        │           │          │          │          │          │
 │          │        │           │          │          │  message_id         │
 │          │        │◄─────────────────────┼──────────┼──────────          │
 │          │        │           │          │          │          │          │
 │          │        │ log_audit │          │          │          │          │
 │          │        │──────────────────────┼──────────┼──────────┼─────────►
 │          │        │           │          │          │          │          │
 │          │        │  ✓ success│          │          │          │          │
 │          │        │◄─────────────────────┼──────────┼──────────┼──────────
 │          │        │           │          │          │          │          │
 │          │ response│          │          │          │          │          │
 │          │◄────────           │          │          │          │          │
 │          │        │           │          │          │          │          │
 │ Display  │        │           │          │          │          │          │
 │ message  │        │           │          │          │          │          │
 │◄─────────         │           │          │          │          │          │
 │          │        │           │          │          │          │          │
```

**Key Points:**
1. Consent check happens FIRST (gate-keeping)
2. PII detection runs BEFORE encryption
3. Only encrypted data touches database
4. Audit log created regardless of outcome

---

## 3. Model Loading Flow (Sequence Diagram)

```
User    ModelUI   Tauri    Model Card   Transparency   Database   LLM
 │        │        │       Fetcher         State         │      Manager
 │        │        │          │               │          │         │
 │ Select │        │          │               │          │         │
 │ model  │        │          │               │          │         │
 │────────►        │          │               │          │         │
 │        │        │          │               │          │         │
 │        │ load_  │          │               │          │         │
 │        │ model  │          │               │          │         │
 │        │────────►          │               │          │         │
 │        │        │          │               │          │         │
 │        │        │ fetch_   │               │          │         │
 │        │        │ model    │               │          │         │
 │        │        │ _card    │               │          │         │
 │        │        │─────────►                │          │         │
 │        │        │          │               │          │         │
 │        │        │   (async download)       │          │         │
 │        │        │          │               │          │         │
 │        │        │ check_ack│               │          │         │
 │        │        │──────────┼───────────────┼─────────►         │
 │        │        │          │               │          │         │
 │        │        │      not_acknowledged    │          │         │
 │        │        │◄─────────────────────────┼──────────         │
 │        │        │          │               │          │         │
 │        │ Show   │          │               │          │         │
 │        │disclaimer│        │               │          │         │
 │        │modal   │          │               │          │         │
 │◄───────         │          │               │          │         │
 │        │        │          │               │          │         │
 │ Click  │        │          │               │          │         │
 │ "I     │        │          │               │          │         │
 │ understand"     │          │               │          │         │
 │────────►        │          │               │          │         │
 │        │        │          │               │          │         │
 │        │ ack_   │          │               │          │         │
 │        │disclaimer│        │               │          │         │
 │        │────────►          │               │          │         │
 │        │        │          │               │          │         │
 │        │        │ store_ack│               │          │         │
 │        │        │──────────┼───────────────┼─────────►         │
 │        │        │          │               │          │         │
 │        │        │  ✓ stored│               │          │         │
 │        │        │◄─────────────────────────┼──────────         │
 │        │        │          │               │          │         │
 │        │        │ create_transparency_ctx  │          │         │
 │        │        │──────────┼───────────────►          │         │
 │        │        │          │               │          │         │
 │        │        │ load_model_weights       │          │         │
 │        │        │──────────────────────────┼──────────┼────────►
 │        │        │          │               │          │         │
 │        │        │              ✓ loaded    │          │         │
 │        │        │◄─────────────────────────┼──────────┼─────────
 │        │        │          │               │          │         │
 │        │ Ready  │          │               │          │         │
 │◄───────         │          │               │          │         │
 │        │        │          │               │          │         │
```

**Key Points:**
1. Model card fetched asynchronously (non-blocking)
2. Disclaimer shown ONLY if not previously acknowledged
3. Acknowledgment stored per-user per-model
4. Transparency context created for AI Act compliance

---

## 4. Setup Wizard Flow (State Diagram)

```
┌────────────┐
│   Start    │
│  (Fresh    │
│  Install)  │
└─────┬──────┘
      │
      ▼
┌────────────────┐
│  Step 1:       │
│  Welcome &     │──► Show GDPR summary
│  Privacy       │    Show data usage overview
│  Notice        │
└────────┬───────┘
      │
      ▼
┌────────────────┐
│  Step 2:       │
│  Consent       │──► Collect all consent types
│  Collection    │    Required: chat, pii, ai_processing
│                │    Optional: analytics, document
└────────┬───────┘
      │
      ▼
┌────────────────┐
│  Step 3:       │
│  PII           │──► Detect system RAM
│  Configuration │    Recommend PII mode
│                │    Show Presidio install guide
└────────┬───────┘
      │
      ▼
┌────────────────┐
│  Step 4:       │
│  Chat          │──► Set encryption password/PIN
│  Encryption    │    Choose auto-unlock preference
│  Setup         │    Generate encryption key
└────────┬───────┘
      │
      ▼
┌────────────────┐
│  Step 5:       │
│  Data          │──► Set retention periods
│  Retention     │    Configure auto-cleanup
│  Preferences   │    Choose deletion schedule
└────────┬───────┘
      │
      ▼
┌────────────────┐
│  Step 6:       │
│  Summary &     │──► Review all settings
│  Confirmation  │    Provide links to change later
│                │    Mark setup complete
└────────┬───────┘
      │
      ▼
┌────────────────┐
│  Setup         │
│  Complete      │──► Store completion flag
│                │    Initialize services
│  ✓             │    Navigate to main app
└────────────────┘
```

**State Transitions:**
- Each step can go FORWARD or BACK (except step 1)
- Step 2 (Consents) cannot be skipped (required)
- Steps 3-5 can be skipped (defaults used)
- Step 6 must be confirmed to complete setup

---

## 5. PII Detection Mode Selection (Decision Tree)

```
                    Detect System Memory
                            │
              ┌─────────────┼─────────────┐
              │             │             │
          < 4GB         4-8GB         > 8GB
              │             │             │
              ▼             ▼             ▼
        Built-in Only  Check Presidio  Check Presidio
              │             │             │
              │      ┌──────┴──────┐      │
              │      │             │      │
              │  Installed    Not Installed│
              │      │             │      │
              │      ▼             │      ▼
              │  Presidio         │   Full Presidio
              │   Lite            │      │
              │      │             │      │
              │      │             ▼      │
              │      │      Show Install  │
              │      │      Guide + Use   │
              │      │      Built-in      │
              │      │             │      │
              └──────┴─────────────┴──────┘
                          │
                          ▼
              ┌───────────────────────┐
              │   Initialize Selected │
              │        PII Mode       │
              └───────────────────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │  Update Status in UI  │
              │  Memory Usage: XMB    │
              └───────────────────────┘
```

**Fallback Strategy:**
```
Runtime PII Detection
         │
         ▼
    Try Presidio
         │
    ┌────┴────┐
    │         │
  Success   Failure
    │         │
    │         ▼
    │    Log Error
    │         │
    │         ▼
    │    Fall back to
    │     Built-in
    │         │
    └─────┬───┘
          │
          ▼
    Process Document
```

---

## 6. Data Flow Architecture

### 6.1 Chat Message Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    User Input Layer                         │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ (plaintext message)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  Consent Validation Layer                    │
│  - Check user consent for chat storage                      │
│  - Verify consent version is up-to-date                     │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ (authorized)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  PII Detection Layer                         │
│  - Scan for sensitive information                           │
│  - Redact detected PII                                      │
│  - Log PII statistics                                       │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ (cleaned text)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  Encryption Layer                            │
│  - Fetch user encryption key from keychain                  │
│  - Encrypt message with AES-256-GCM                         │
│  - Generate authentication tag                              │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ (encrypted blob)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  Database Storage Layer                      │
│  - Store encrypted message                                  │
│  - Store metadata (timestamp, model, user_id)               │
│  - Link to conversation thread                              │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ (message_id)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  Audit Logging Layer                         │
│  - Log operation success                                    │
│  - Record encryption status                                 │
│  - Timestamp and user info                                  │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Message Retrieval Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  Database Query Layer                        │
│  - Fetch encrypted messages for conversation                │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ (encrypted messages)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  Decryption Layer                            │
│  - Fetch user decryption key from keychain                  │
│  - Decrypt each message                                     │
│  - Verify authentication tag                                │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        │ (plaintext messages)
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  Display Layer                               │
│  - Render messages in UI                                    │
│  - Show confidence indicators                               │
│  - Display timestamps                                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Component Dependencies

```
┌──────────────────────────────────────────────────────────────┐
│                         AppState                             │
└────────┬────────────────────────────────────────────┬────────┘
         │                                            │
    ┌────▼────┐                                  ┌────▼────┐
    │ Consent │                                  │  Chat   │
    │  Guard  │                                  │Encryption│
    └────┬────┘                                  └────┬────┘
         │                                            │
         │ depends on                    depends on  │
         │                                            │
    ┌────▼────────┐                          ┌───────▼──────┐
    │ Compliance  │                          │    Key       │
    │   Manager   │                          │  Manager     │
    └────┬────────┘                          └───────┬──────┘
         │                                            │
         │ uses                              uses     │
         │                                            │
    ┌────▼────────┐                          ┌───────▼──────┐
    │  Consent    │                          │  SQLCipher   │
    │  Manager    │                          │   Database   │
    └─────────────┘                          └──────────────┘


┌──────────────────────────────────────────────────────────────┐
│                         AppState                             │
└────────┬────────────────────────────────────────────┬────────┘
         │                                            │
    ┌────▼────┐                                  ┌────▼────┐
    │   PII   │                                  │ Model   │
    │Detector │                                  │  Card   │
    │         │                                  │Fetcher  │
    └────┬────┘                                  └────┬────┘
         │                                            │
         │ can use                       fetches     │
         │                                            │
    ┌────▼────────┐                          ┌───────▼──────┐
    │  Presidio   │                          │ HuggingFace  │
    │   Bridge    │                          │     API      │
    └─────────────┘                          └──────────────┘
```

**Initialization Order:**
1. Database Manager (first - needed by everyone)
2. Key Manager (encryption keys)
3. Consent Manager (gates all operations)
4. PII Detector (optional Presidio)
5. Chat Encryption (depends on Key Manager)
6. Transparency State
7. Model Card Fetcher (async, non-critical)
8. Consent Guard (coordinates above)

---

## 8. Error Propagation Flow

```
Operation Initiated
        │
        ▼
┌───────────────┐
│ Consent Check │──► Denied ──► Show Consent Modal ──► Retry
└───────┬───────┘                                        │
        │                                                │
        │ Granted                                        │
        ▼                                                │
┌───────────────┐                                        │
│ PII Detection │──► Error ──► Fallback to Built-in ───►│
└───────┬───────┘                                        │
        │                                                │
        │ Success                                        │
        ▼                                                │
┌───────────────┐                                        │
│  Encryption   │──► Error ──► Block + Show Error ──────┘
└───────┬───────┘
        │
        │ Success
        ▼
┌───────────────┐
│   Database    │──► Error ──► Rollback + Log ──────────┐
└───────┬───────┘                                        │
        │                                                │
        │ Success                                        │
        ▼                                                │
┌───────────────┐                                        │
│ Audit Logging │──► Error ──► Log to stderr ───────────┘
└───────┬───────┘
        │
        │ Success
        ▼
Return Success to User
```

---

## 9. Security Boundaries

```
┌────────────────────────────────────────────────────────┐
│              Untrusted Zone (User Input)               │
│  • User-entered messages                               │
│  • Uploaded documents                                  │
│  • External model cards                                │
└───────────────────────┬────────────────────────────────┘
                        │
                        │ (validation + sanitization)
                        ▼
┌────────────────────────────────────────────────────────┐
│           Application Trust Boundary                   │
│  • PII Detection (sanitization)                        │
│  • Input validation                                    │
│  • Consent verification                                │
└───────────────────────┬────────────────────────────────┘
                        │
                        │ (encrypted)
                        ▼
┌────────────────────────────────────────────────────────┐
│              Data at Rest (Encrypted)                  │
│  • Encrypted chat messages                             │
│  • Encrypted documents                                 │
│  • Consent records (plaintext for compliance)          │
│  • Audit logs (plaintext for compliance)               │
└────────────────────────────────────────────────────────┘
```

**Key Security Principles:**
1. **Defense in Depth**: Multiple validation layers
2. **Least Privilege**: Encryption keys stored in OS keychain
3. **Fail Secure**: Operations blocked on consent/encryption failure
4. **Audit Everything**: All data operations logged
5. **Encryption by Default**: All user content encrypted before storage

---

## 10. Performance Considerations

### Critical Path Latency Budget

```
User sends message
    ↓
Consent check (DB query)          ~5-10ms
    ↓
PII detection                     ~50-200ms (Presidio)
                                  ~10-20ms (Built-in)
    ↓
Encryption                        ~5-10ms
    ↓
Database insert                   ~5-15ms
    ↓
Audit log                         ~5-10ms
    ↓
Total latency                     ~70-245ms

TARGET: < 200ms for good UX
FALLBACK: Use built-in PII if > 200ms
```

### Optimization Strategies

1. **Parallel Operations:**
   - Model card fetch (async, don't block)
   - Audit logging (fire-and-forget)

2. **Caching:**
   - Consent status (30 second TTL)
   - Encryption keys (in-memory)
   - Model disclaimers (per session)

3. **Database:**
   - Connection pooling (reuse connections)
   - Prepared statements (avoid re-parsing)
   - Batch inserts for audit logs

4. **PII Detection:**
   - Size threshold (> 10KB → built-in only)
   - Timeout (> 200ms → fallback)

---

## Conclusion

This architecture provides:
- **Clear separation of concerns** across components
- **Explicit data flows** with security boundaries
- **Graceful degradation** on component failures
- **GDPR and AI Act compliance** by design
- **Performance optimization** opportunities

All components work together to ensure user data is protected while maintaining regulatory compliance.
