# Security & Performance Improvement Roadmap

## 🚨 Priority 1: Critical Security Fixes (Must Fix Immediately)

### 1.1 Path Traversal & Temporary File Security
**Files:** `main.rs`, `file_processor.rs`
**Risk:** High - Arbitrary file system access, file leaks
**Impact:** Security vulnerability, resource exhaustion

**Issues:**
- `create_secure_temp_path` (main.rs:119-125) doesn't canonicalize paths
- `TempFileGuard` can leak files if operations fail after guard creation
- No validation that temporary files stay within temp directory

**Fix Plan:**
```rust
// Add canonicalization and validation
fn create_secure_temp_path(original_filename: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let sanitized = original_filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_')
        .collect::<String>();

    let temp_path = temp_dir.join(format!("bear_ai_{}", sanitized));

    // Canonicalize to prevent path traversal
    let canonical = temp_path.canonicalize()
        .unwrap_or_else(|_| temp_path.clone());

    // Ensure path is within temp directory
    if !canonical.starts_with(&temp_dir) {
        return Err(anyhow!("Path traversal attempt detected"));
    }

    Ok(canonical)
}
```

**Estimated Time:** 2-3 hours
**Testing:** Path traversal test suite

---

### 1.2 Python/SQL Sandbox Escape
**Files:** `mcp_server.rs`
**Risk:** Critical - Remote code execution
**Impact:** Complete system compromise

**Issues:**
- Regex-based filtering for `os`, `subprocess` can be bypassed
- SQL injection still possible despite SELECT-only filtering
- No process isolation or capability restrictions

**Fix Plan:**
1. **Immediate:** Disable Python/SQL execution entirely until sandboxed
2. **Short-term:** Use PyO3 with restricted builtins
3. **Long-term:** Move to containerized execution (Docker/Podman)

**Recommended Approach:**
```rust
// Replace execute_python with PyO3 safe mode
pub async fn execute_python_safe(code: &str) -> Result<String> {
    use pyo3::prelude::*;

    Python::with_gil(|py| {
        // Create restricted environment
        let restricted = PyDict::new(py);
        restricted.set_item("__builtins__", py.None())?;

        // Add only safe modules
        let safe_modules = ["math", "json", "datetime"];
        for module in safe_modules {
            let m = py.import(module)?;
            restricted.set_item(module, m)?;
        }

        py.eval(code, Some(restricted), None)
            .map(|r| r.to_string())
    })
}
```

**Estimated Time:** 1-2 days
**Testing:** Fuzzing with malicious payloads

---

### 1.3 Path Canonicalization in MCP Server
**Files:** `mcp_server.rs`
**Risk:** Medium-High - Unauthorized file access
**Impact:** Data breach, file manipulation

**Issues:**
- `is_path_allowed` checks string prefixes, not canonical paths
- Symbolic links and `..` can escape allowed directories

**Fix:**
```rust
fn is_path_allowed(path: &Path, allowed_paths: &[PathBuf]) -> bool {
    // Canonicalize both target and allowed paths
    let canonical_target = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return false, // Non-existent paths are denied
    };

    allowed_paths.iter().any(|allowed| {
        if let Ok(canonical_allowed) = allowed.canonicalize() {
            canonical_target.starts_with(&canonical_allowed)
        } else {
            false
        }
    })
}
```

**Estimated Time:** 1 hour
**Testing:** Symbolic link and `..` escape tests

---

## ⚠️ Priority 2: Stability & Reliability (Fix Soon)

### 2.1 Model Path Inconsistency
**Files:** `llm_manager.rs:321-322, 378-380`
**Risk:** Low - Feature breakage
**Impact:** Models fail to load after download

**Issue:**
- `load_model` uses `repo_id.replace("/", "_")`
- `download_model` uses `model_name`
- Models downloaded by one method are invisible to the other

**Fix:**
```rust
// Standardize on sanitized repo_id
fn get_model_dir(&self, model_config: &ModelConfig) -> PathBuf {
    let sanitized = model_config.repo_id.replace("/", "_");
    self.models_dir.join(sanitized)
}

// Use in both load_model and download_model
let model_dir = self.get_model_dir(model_config);
```

**Estimated Time:** 30 minutes
**Testing:** Download → Load cycle test

---

### 2.2 Missing Sampling Configuration
**Files:** `llm_manager.rs:425-427, 497-499`
**Risk:** Low - Poor output quality
**Impact:** Non-creative, deterministic responses

**Issue:**
- Only uses `sample_token_greedy`
- Ignores `temperature`, `top_k`, `top_p` configuration

**Fix:**
```rust
// In generate and generate_stream methods
let sampled_token = if config.temperature > 0.0 {
    // Apply temperature scaling
    for i in 0..candidates_array.data.len() {
        candidates_array.data[i].p =
            (candidates_array.data[i].p / config.temperature).exp();
    }

    // Apply top_k filtering
    ctx.sample_top_k(&mut candidates_array, config.top_k, 1);

    // Apply top_p (nucleus) sampling
    ctx.sample_top_p(&mut candidates_array, config.top_p, 1);

    // Sample token
    ctx.sample_token(&mut candidates_array)
} else {
    // Greedy decoding when temperature = 0
    ctx.sample_token_greedy(&mut candidates_array)
};
```

**Estimated Time:** 2 hours
**Testing:** Compare outputs with different temperature values

---

### 2.3 Token Overflow Handling
**Files:** `gguf_inference.rs:195-200`
**Risk:** Low - Runtime errors
**Impact:** Generation fails with long prompts

**Issue:**
- No check if prompt exceeds `n_ctx`
- Can cause decode failures or undefined behavior

**Fix:**
```rust
// Check token count before creating batch
let mut tokens = model.str_to_token(prompt, AddBos::Always)?;

// Truncate if necessary
if tokens.len() > config.n_ctx as usize - max_tokens {
    let max_prompt_tokens = config.n_ctx as usize - max_tokens - 10; // Safety margin
    tokens.truncate(max_prompt_tokens);

    tracing::warn!(
        "Prompt truncated from {} to {} tokens to fit context",
        tokens.len(),
        max_prompt_tokens
    );
}
```

**Estimated Time:** 1 hour
**Testing:** Generate with prompts exceeding context length

---

### 2.4 Race Conditions in Setup
**Files:** `setup_manager.rs`
**Risk:** Low - Setup corruption
**Impact:** Failed initialization on concurrent launches

**Issue:**
- No lock preventing simultaneous setup runs
- Directory creation and marker file writes are not atomic

**Fix:**
```rust
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

static SETUP_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub async fn run_initial_setup(
    &mut self,
    config: SetupConfig,
    progress_callback: Option<impl Fn(f32) + Send + Sync + 'static>,
) -> Result<()> {
    // Acquire lock to prevent concurrent setup
    let _lock = SETUP_LOCK.lock().await;

    // Check marker file again after acquiring lock
    if self.is_setup_complete().await {
        return Ok(());
    }

    // ... rest of setup logic
}
```

**Estimated Time:** 1 hour
**Testing:** Concurrent launch stress test

---

## 🔧 Priority 3: Performance & Maintainability (Nice to Have)

### 3.1 Replace wmic with Windows API
**Files:** `system_monitor.rs`
**Risk:** None - Future deprecation
**Impact:** GPU detection may fail on future Windows

**Fix:**
- Add `windows-rs` crate
- Use WMI queries directly via COM APIs
- Fallback to current wmic if API fails

**Estimated Time:** 4-6 hours
**Benefit:** More robust, faster, no process spawning

---

### 3.2 Centralize Magic Numbers
**Files:** `llm_manager.rs`, `hardware_monitor.rs`, `system_monitor.rs`
**Risk:** None
**Impact:** Hard to maintain, inconsistent thresholds

**Create:** `src-tauri/src/constants.rs`
```rust
// Hardware thresholds
pub const MIN_VRAM_MB: u64 = 2048;
pub const GPU_TEMP_WARNING: f32 = 75.0;
pub const GPU_TEMP_CRITICAL: f32 = 85.0;
pub const VRAM_USAGE_RATIO: f32 = 0.8;

// Model defaults
pub const DEFAULT_N_CTX: u32 = 4096;
pub const DEFAULT_N_BATCH: u32 = 512;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;
```

**Estimated Time:** 2 hours
**Benefit:** Single source of truth, easier tuning

---

### 3.3 Configurable PII Exclusions
**Files:** `pii_detector.rs`
**Risk:** None
**Impact:** False positives in different jurisdictions

**Fix:**
- Move exclusion lists to TOML config file
- Load at startup via serde
- Allow runtime updates via command

**Estimated Time:** 3 hours
**Benefit:** No recompilation for PII rules

---

### 3.4 Optimize System Monitoring
**Files:** `system_monitor.rs`
**Risk:** None
**Impact:** Unnecessary CPU overhead

**Fix:**
- Use targeted refresh methods
- Cache results with TTL
- Only refresh changed metrics

**Estimated Time:** 2 hours
**Benefit:** Lower CPU usage in background monitoring

---

## 📊 Summary

| Priority | Category | Items | Time Estimate | Risk Level |
|----------|----------|-------|---------------|------------|
| P1 | Security | 3 | 3-5 days | Critical/High |
| P2 | Stability | 4 | 1 day | Low-Medium |
| P3 | Performance | 4 | 2-3 days | None |
| **Total** | | **11** | **6-9 days** | |

## 🎯 Recommended Implementation Order

### Week 1 (Critical Security)
1. ✅ Disable Python/SQL execution (immediate)
2. 🔧 Fix path canonicalization (main.rs, mcp_server.rs)
3. 🔧 Fix TempFileGuard leaks

### Week 2 (Stability)
4. 🔧 Fix model path inconsistency
5. 🔧 Add token overflow handling
6. 🔧 Implement sampling configuration
7. 🔧 Add setup race condition protection

### Week 3+ (Performance)
8. 🔧 Replace wmic with Windows API
9. 🔧 Centralize constants
10. 🔧 Make PII exclusions configurable
11. 🔧 Optimize system monitoring

---

**Status:** Draft
**Last Updated:** 2025-10-01
**Next Review:** After P1 completion
