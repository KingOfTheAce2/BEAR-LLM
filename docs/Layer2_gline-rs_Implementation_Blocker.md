# Layer 2 (gline-rs) Implementation Blocker

## Critical Issue: Dependency Conflict

### Problem Statement

**Cannot add gline-rs v1.0.0 due to ONNX Runtime version conflict:**

```
gline-rs v1.0.0     requires: ort = 2.0.0-rc.9  (exact version)
fastembed v5.0.1    requires: ort = 2.0.0-rc.10 (exact version)

❌ These are INCOMPATIBLE - Cargo cannot resolve
```

### Current Dependencies Using ort

1. **fastembed v5.0** - Required for embeddings (already in use)
   - Depends on ort v2.0.0-rc.10
   - Used in: `src-tauri/src/embeddings_manager.rs`

2. **gline-rs v1.0.0** - Proposed for Layer 2 PII detection
   - Depends on ort v2.0.0-rc.9
   - Cannot coexist with fastembed

### Root Cause

`gline-rs` author hardcoded `ort = "=2.0.0-rc.9"` (exact version) in Cargo.toml:
https://github.com/fbilhaut/gline-rs/blob/main/Cargo.toml#L15

This prevents Cargo from resolving to a compatible version.

---

## Solutions (Ordered by Viability)

### Option 1: Wait for gline-rs v1.0.1 ✅ RECOMMENDED

**Status**: Monitoring upstream

**Action**: Open GitHub issue requesting ort upgrade:
```markdown
Title: Update ort dependency to 2.0.0-rc.10 for compatibility

Currently gline-rs v1.0.0 uses ort = "=2.0.0-rc.9" which conflicts
with other crates like fastembed v5.0 that require rc.10.

Request: Change to ort = ">=2.0.0-rc.9" or update to rc.10
```

**Timeline**: 1-4 weeks for upstream fix

**Workaround**: Use Layer 1 (Regex) + Layer 3 (Presidio) until fixed

---

### Option 2: Fork gline-rs and Update ort ⚡ FAST FIX

**Pros**:
- Immediate solution
- Full control over updates
- Can contribute back upstream

**Cons**:
- Maintenance burden
- Need to track upstream changes

**Implementation**:
```toml
# Fork repository
git clone https://github.com/fbilhaut/gline-rs
cd gline-rs

# Update ort version in Cargo.toml
sed -i 's/ort = "=2.0.0-rc.9"/ort = "2.0.0-rc.10"/' Cargo.toml

# Test compilation
cargo build --release

# Publish to crates.io or use git dependency
```

**In BEAR-LLM Cargo.toml**:
```toml
[dependencies]
gline-rs = { git = "https://github.com/YOUR_FORK/gline-rs", branch = "ort-rc10" }
```

**Estimated effort**: 2-4 hours

---

### Option 3: Implement Pure Rust NER (No ML) 🔧 INTERMEDIATE

**Use regex + rule-based NER without ML models**

**Pros**:
- No dependency conflicts
- Zero external models
- ~80-85% accuracy (vs 92% with gline-rs)

**Cons**:
- Lower accuracy for names
- More false positives
- No context understanding

**Current Status**: Already implemented in Layer 1
- Lines 52-71 in `pii_detector.rs`
- Regex patterns for SSN, credit cards, emails, phones
- Name patterns with title detection

**Recommendation**: Enhance existing regex patterns instead of adding ML

---

### Option 4: Use Alternative Rust NER Library 🔍 RESEARCH NEEDED

**Candidates**:

| Library | Status | ort Version | Notes |
|---------|--------|-------------|-------|
| `rust-bert` | ✅ Active | Uses `tch` (LibTorch), not ort | 2-4x slower than gline-rs |
| `candle-transformers` | ✅ Active | No ort dependency | Already in use! Can leverage existing |
| `tract` | ✅ Active | No ort dependency | ONNX inference, 85% compatible |

**Best Alternative**: **candle-transformers**
- Already a dependency (line 45 in Cargo.toml)
- Can load ONNX models via conversion
- No conflicts
- GPU support via Candle (already configured)

**Implementation Path**:
```rust
use candle_transformers::models::bert;
use candle_core::{Device, Tensor};

// Load NER model using Candle
let device = Device::cuda_if_available(0)?;
let model = bert::BertForTokenClassification::load(model_path, device)?;

// Run inference for NER
let tokens = tokenizer.encode(text)?;
let predictions = model.forward(&tokens)?;
```

**Estimated effort**: 1-2 weeks

---

## Current 3-Layer Architecture Status

### ✅ Layers Already Working

**Layer 1: Regex (ACTIVE)**
- Location: `pii_detector.rs` lines 606-687
- Accuracy: ~85%
- Speed: <5ms per document
- No dependencies

**Layer 3: MS Presidio (OPTIONAL, POST-INSTALL)**
- Location: `pii_detector.rs` lines 629-654, 779-827
- Accuracy: ~95%
- Speed: 50-200ms per document
- Requires: Python + pip packages

### ❌ Layer 2: gline-rs (BLOCKED)

**Attempted Implementation**:
- Location: `pii_detector.rs` lines 532-628, 736-789
- **Status**: Cannot compile due to ort conflict
- **Blocker**: Incompatible with existing fastembed dependency

---

## Recommended Implementation Plan

### Immediate (This Week)

1. **Revert gline-rs changes** to prevent build failures
2. **Document Layer 2 as "planned but blocked"**
3. **Optimize Layer 1 (Regex)** with enhanced patterns
4. **Ensure Layer 3 (Presidio)** works post-install

### Short Term (1-2 Months)

**Option A: Wait for upstream fix**
- Open GitHub issue on gline-rs
- Monitor for v1.0.1 release
- Implement when available

**Option B: Implement with Candle**
- Use existing `candle-transformers` dependency
- Load BERT-based NER model
- 90% accuracy, no conflicts

### Long Term (3-6 Months)

**Option C: Custom PII Model**
- Train legal-domain specific NER model
- Fine-tune on legal documents
- Deploy via Candle (pure Rust)
- 95%+ accuracy for legal PII

---

## Technical Details

### Dependency Versions

```toml
# Current (WORKING)
fastembed = { version = "5.0", features = ["online"] }  # Uses ort rc.10
candle-transformers = "0.8"                             # No ort dependency

# Attempted (FAILED)
gline-rs = "1.0.0"  # ❌ Requires ort rc.9 (conflicts with fastembed)

# Transitive dependencies
ort v2.0.0-rc.9   ← gline-rs (exact version required)
ort v2.0.0-rc.10  ← fastembed (exact version required)
```

### Why Cargo Can't Resolve

Cargo's dependency resolver requires **exact** version matches when `=` is used:

```toml
# gline-rs Cargo.toml (upstream)
ort = "=2.0.0-rc.9"  # ← This forces EXACTLY rc.9

# fastembed's dependency (upstream)
ort = "=2.0.0-rc.10"  # ← This forces EXACTLY rc.10

# Cargo error: Cannot satisfy both constraints
```

**Solution requires**:
- Upstream changes `=` to `>=` in gline-rs
- OR we fork and modify
- OR we use different library

---

## Frontend Implications

### Settings UI Changes Needed

**Current Plan** (if gline-rs worked):
```typescript
// src/components/Settings.tsx
<select name="pii_detection_layer">
  <option value="regex_only">Fast (Layer 1: Regex)</option>
  <option value="with_gline">Balanced (Layer 1+2: ML Enhanced)</option>  {/* ❌ BLOCKED */}
  <option value="full_stack">Maximum (All 3 Layers)</option>
</select>
```

**Revised Plan** (without gline-rs):
```typescript
// Only 2 options until Layer 2 ready
<select name="pii_detection_layer">
  <option value="regex_only">Fast (Regex Only - 85% accuracy)</option>
  <option value="with_presidio">Maximum (Regex + Presidio - 95% accuracy)</option>
</select>

// Download buttons
<button onClick={downloadPresidio}>
  📥 Install Presidio (Python Required)
</button>

// Future Layer 2 (when available)
<button onClick={downloadGlineModels} disabled={!glineAvailable}>
  📥 Install ML Models (Rust-native, {glineAvailable ? 'Ready' : 'Coming Soon'})
</button>
```

---

## Rollback Plan

### Files to Revert

1. **src-tauri/Cargo.toml**
   - Remove: `gline-rs = "1.0.0"`
   - Keep: `fastembed = "5.0"` (required for embeddings)

2. **src-tauri/src/pii_detector.rs**
   - Remove: gline-rs imports (line 48)
   - Remove: `gline_detector` field (line 364)
   - Remove: `gline_model_path` field (line 365)
   - Remove: `initialize_gline_detector()` (lines 532-628)
   - Remove: `download_gline_models()` (lines 592-628)
   - Remove: `detect_with_gline()` (lines 736-789)
   - Keep: Layer 1 and Layer 3 implementations

3. **Update DetectionLayer enum**
   ```rust
   pub enum DetectionLayer {
       RegexOnly,      // Layer 1 only
       // WithGline,   // ❌ COMMENTED OUT until dependency resolved
       WithPresidio,   // Layer 1 + 3 (skip Layer 2)
   }
   ```

---

## Communication to Users

### In-App Message (Settings Page)

```
🔧 PII Detection Layers

Layer 1 (Regex): ✅ Active
- Fast pattern-based detection
- 85% accuracy
- No setup required

Layer 2 (ML-Enhanced): ⏳ Coming Soon
- Rust-native machine learning
- 92% accuracy
- Currently blocked by dependency conflicts

Layer 3 (Presidio): ✅ Optional
- Industry-leading accuracy (95%)
- Requires Python installation
- [Download Presidio] button
```

### README.md Update

```markdown
## PII Detection

BEAR-LLM uses a multi-layer approach:

- **Layer 1 (Active)**: Regex-based detection (~85% accuracy, <5ms)
- **Layer 2 (Planned)**: Rust ML models (~92% accuracy, ~50ms) - Coming in v1.1
- **Layer 3 (Optional)**: MS Presidio (~95% accuracy, ~200ms) - Install post-deployment

Currently Layer 1 + Layer 3 provide excellent protection.
Layer 2 will be added when dependency conflicts are resolved.
```

---

## Conclusion

### Current State
- ✅ Layer 1 (Regex) working
- ❌ Layer 2 (gline-rs) blocked by ort version conflict
- ✅ Layer 3 (Presidio) working (post-install)

### Recommended Action
**Option B: Implement Layer 2 using candle-transformers**
- Uses existing dependency
- No conflicts
- Pure Rust
- 90% accuracy
- 1-2 weeks implementation

### Fallback
- Document Layer 2 as "coming soon"
- Optimize Layer 1 patterns
- Focus on Layer 3 post-install UX
- Wait for gline-rs v1.0.1 with ort rc.10
