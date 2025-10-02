/// Application-wide constants
///
/// This module centralizes all magic numbers, thresholds, and configuration values
/// for easier maintenance and tuning.
// ============================================================================
// Hardware Monitoring Thresholds
// ============================================================================
/// Minimum VRAM required for GPU acceleration (in MB)
pub const MIN_VRAM_MB: u64 = 2048;

/// GPU temperature warning threshold (in Celsius)
pub const GPU_TEMP_WARNING: f32 = 75.0;

/// GPU temperature critical threshold (in Celsius)
pub const GPU_TEMP_CRITICAL: f32 = 85.0;

/// CPU temperature warning threshold (in Celsius)
pub const CPU_TEMP_WARNING: f32 = 80.0;

/// CPU temperature critical threshold (in Celsius)
pub const CPU_TEMP_CRITICAL: f32 = 90.0;

/// VRAM usage ratio to reserve for system stability (80% usable)
pub const VRAM_USAGE_RATIO: f32 = 0.8;

/// Number of consecutive high temperature readings before triggering safety measures
pub const CONSECUTIVE_HIGH_READINGS_LIMIT: usize = 3;

/// System monitoring refresh interval (in milliseconds)
pub const SYSTEM_MONITOR_REFRESH_MS: u64 = 1000;

// ============================================================================
// Model Configuration Defaults
// ============================================================================

/// Default context length for LLM inference
pub const DEFAULT_N_CTX: u32 = 4096;

/// Default batch size for prompt processing
pub const DEFAULT_N_BATCH: u32 = 512;

/// Default sampling temperature
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Default top-k sampling parameter
pub const DEFAULT_TOP_K: i32 = 40;

/// Default top-p (nucleus) sampling parameter
pub const DEFAULT_TOP_P: f32 = 0.9;

/// Default repetition penalty
pub const DEFAULT_REPEAT_PENALTY: f32 = 1.1;

/// Maximum tokens to generate per request
pub const DEFAULT_MAX_TOKENS: usize = 2048;

/// Safety margin for token overflow prevention (in tokens)
pub const TOKEN_OVERFLOW_SAFETY_MARGIN: usize = 10;

// ============================================================================
// RAG Engine Configuration
// ============================================================================

/// Default number of documents to retrieve for RAG
pub const RAG_DEFAULT_TOP_K: usize = 5;

/// Minimum similarity score for RAG document retrieval
pub const RAG_MIN_SIMILARITY: f32 = 0.3;

/// Maximum chunk size for document splitting (in characters)
pub const RAG_MAX_CHUNK_SIZE: usize = 1000;

/// Chunk overlap size for context preservation (in characters)
pub const RAG_CHUNK_OVERLAP: usize = 200;

// ============================================================================
// Embedding Model Configuration
// ============================================================================

/// Default embedding model name
pub const DEFAULT_EMBEDDING_MODEL: &str = "BAAI/bge-small-en-v1.5";

/// BGE-Small embedding dimension
pub const BGE_SMALL_DIMENSION: usize = 384;

/// BGE-Base embedding dimension
pub const BGE_BASE_DIMENSION: usize = 768;

/// All-MiniLM-L6-v2 embedding dimension
pub const ALL_MINILM_L6_DIMENSION: usize = 384;

// ============================================================================
// Database Configuration
// ============================================================================

/// Maximum number of documents to return in search
pub const DB_MAX_SEARCH_RESULTS: usize = 100;

/// Database connection pool size
pub const DB_POOL_SIZE: u32 = 5;

/// Database query timeout (in seconds)
pub const DB_QUERY_TIMEOUT_SECS: u64 = 30;

// ============================================================================
// PII Detection Configuration
// ============================================================================

/// Minimum confidence score for PII detection
pub const PII_MIN_CONFIDENCE: f32 = 0.5;

/// High confidence threshold for automatic redaction
pub const PII_HIGH_CONFIDENCE: f32 = 0.85;

/// Context window size for PII detection (in characters)
pub const PII_CONTEXT_WINDOW: usize = 100;

// ============================================================================
// File Processing Limits
// ============================================================================

/// Maximum file size for processing (in MB)
pub const MAX_FILE_SIZE_MB: usize = 100;

/// Maximum number of pages to extract from PDF
pub const MAX_PDF_PAGES: usize = 1000;

/// Maximum text extraction length (in characters)
pub const MAX_TEXT_EXTRACTION_CHARS: usize = 10_000_000; // 10M chars

// ============================================================================
// Network and API Configuration
// ============================================================================

/// HTTP request timeout (in seconds)
pub const HTTP_TIMEOUT_SECS: u64 = 30;

/// Maximum retries for failed requests
pub const MAX_HTTP_RETRIES: usize = 3;

/// Retry backoff delay (in milliseconds)
pub const RETRY_BACKOFF_MS: u64 = 1000;

/// HuggingFace API rate limit delay (in milliseconds)
pub const HF_API_RATE_LIMIT_MS: u64 = 100;

// ============================================================================
// Temporary File Management
// ============================================================================

/// Temporary file prefix
pub const TEMP_FILE_PREFIX: &str = "bear_ai";

/// Temporary file cleanup age (in seconds)
pub const TEMP_FILE_CLEANUP_AGE_SECS: u64 = 3600; // 1 hour

// ============================================================================
// GPU Layer Offloading
// ============================================================================

/// Default GPU layers for TinyLlama 1.1B
pub const TINYLLAMA_GPU_LAYERS: u32 = 22;

/// Default GPU layers for Phi-2 2.7B
pub const PHI2_GPU_LAYERS: u32 = 32;

/// Default GPU layers for Mistral 7B
pub const MISTRAL_7B_GPU_LAYERS: u32 = 35;

/// Default GPU layers for Llama 2 7B
pub const LLAMA2_7B_GPU_LAYERS: u32 = 33;

// ============================================================================
// Memory Requirements (in MB)
// ============================================================================

/// Recommended VRAM for TinyLlama 1.1B (Q4_K_M)
pub const TINYLLAMA_VRAM_MB: u64 = 1024;

/// Recommended VRAM for Phi-2 2.7B (Q4_K_M)
pub const PHI2_VRAM_MB: u64 = 2048;

/// Recommended VRAM for Mistral 7B (Q4_K_M)
pub const MISTRAL_7B_VRAM_MB: u64 = 6144;

/// Recommended VRAM for Llama 2 7B (Q4_K_M)
pub const LLAMA2_7B_VRAM_MB: u64 = 5120;

// ============================================================================
// Application Metadata
// ============================================================================

/// Application name
pub const APP_NAME: &str = "BEAR AI";

/// Application data directory name
pub const APP_DATA_DIR: &str = "bear-ai-llm";

/// Setup completion marker file name
pub const SETUP_MARKER_FILE: &str = ".setup_complete";

/// Configuration file name
pub const CONFIG_FILE: &str = "config.json";

// ============================================================================
// Logging and Monitoring
// ============================================================================

/// Maximum log file size (in MB)
pub const MAX_LOG_FILE_SIZE_MB: usize = 50;

/// Number of log files to retain
pub const LOG_FILE_RETENTION: usize = 5;

/// Metrics collection interval (in seconds)
pub const METRICS_INTERVAL_SECS: u64 = 60;

// ============================================================================
// Performance Tuning
// ============================================================================

/// Maximum concurrent file processing tasks
pub const MAX_CONCURRENT_FILE_TASKS: usize = 4;

/// Thread pool size for CPU-bound tasks
pub const CPU_THREAD_POOL_SIZE: usize = 4;

/// Embedding batch size for bulk processing
pub const EMBEDDING_BATCH_SIZE: usize = 32;

/// Cache size for embeddings (number of entries)
pub const EMBEDDING_CACHE_SIZE: usize = 1000;

// ============================================================================
// Development and Testing
// ============================================================================

/// Enable debug logging in development
pub const DEV_DEBUG_LOGGING: bool = cfg!(debug_assertions);

/// Enable performance profiling
pub const ENABLE_PROFILING: bool = cfg!(debug_assertions);

/// Mock external services in tests
pub const MOCK_EXTERNAL_SERVICES: bool = cfg!(test);
