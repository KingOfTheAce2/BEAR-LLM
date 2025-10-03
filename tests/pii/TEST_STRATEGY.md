# 3-Layer PII Detection Test Strategy

## Overview

Comprehensive test suite for BEAR-LLM's 3-layer PII detection system.

**Test Goal**: 90%+ code coverage across all layers with focus on:
- Layer interactions and fallback mechanisms
- Multi-regional exclusion accuracy (3,474+ patterns)
- Performance benchmarks and concurrent operations
- Error handling and edge cases

---

## Architecture Under Test

### Layer 1: Regex-Based Detection (Baseline)
- **Engine**: Pure Rust regex patterns
- **Accuracy**: ~85%
- **Speed**: <5ms for small documents
- **Coverage**: SSN, emails, phones, credit cards, IPs, case numbers, medical records
- **Always active**: Provides fallback if advanced layers fail

### Layer 2: gline-rs Enhanced Detection (ML-Based)
- **Engine**: Rust-native ML (gline-rs)
- **Accuracy**: ~92%
- **Speed**: 20-40ms for small documents
- **Coverage**: Person names, organizations, locations, context-aware detection
- **Enabled by default**: Can be disabled via configuration

### Layer 3: Microsoft Presidio (Optional Post-Install)
- **Engine**: Python-based NER (spaCy + transformers)
- **Accuracy**: 90-95% (depends on mode)
- **Speed**: 50-200ms for small documents
- **Coverage**: 15+ entity types, enterprise-grade detection
- **Optional**: Requires Python + Presidio installation

---

## Test Suite Structure

```
tests/pii/
├── mod.rs                          # Test module definition
├── pii_layer1_presidio_tests.rs   # Layer 3 (Presidio) tests
├── pii_layer2_regex_tests.rs      # Layer 1 (Regex) tests
├── pii_layer2_gline_tests.rs      # Layer 2 (gline-rs) tests
├── pii_integration_tests.rs       # Cross-layer integration tests
├── pii_exclusions_tests.rs        # Multi-regional exclusion tests
└── TEST_STRATEGY.md                # This document
```

---

## Test Coverage Breakdown

### Layer 1 Tests (pii_layer2_regex_tests.rs)

**Regex Pattern Accuracy** (30 tests)
- ✅ SSN detection (standard format, invalid formats)
- ✅ Credit card detection with Luhn validation
- ✅ Email detection (various formats, special characters)
- ✅ Phone number detection (multiple formats)
- ✅ IP address detection (valid/invalid)
- ✅ Case number detection (legal documents)
- ✅ Medical record number detection
- ✅ Person name detection (capitalized, titled)
- ✅ Organization detection (corporate, law firms)
- ✅ Word boundaries and pattern isolation
- ✅ Confidence scoring for different entity types
- ✅ Selective detection via configuration
- ✅ Thread safety and concurrent requests

**Known Limitations Tested**:
- Lowercase names not detected (regex limitation)
- SSN only matches 123-45-6789 format (not 123456789)
- Name patterns can flag non-names like "Supreme Court"

### Layer 2 Tests (pii_layer2_gline_tests.rs)

**gline-rs ML Detection** (25 tests)
- ✅ Initialization and fallback behavior
- ✅ Entity type mapping (standardization)
- ✅ Person detection (including lowercase names)
- ✅ Organization detection (context-aware)
- ✅ Location detection (multi-language)
- ✅ Email and phone detection (enhanced)
- ✅ Confidence scoring (ML-based)
- ✅ Multilingual support testing
- ✅ Context-awareness validation
- ✅ Performance benchmarks
- ✅ Thread safety and concurrent requests
- ✅ Large document handling
- ✅ Special character handling (Unicode)
- ✅ Entity boundary validation
- ✅ Empty input handling
- ✅ Confidence threshold filtering

**Advantages Over Regex**:
- Detects lowercase names
- Better context awareness
- Higher accuracy for person/organization names
- Multilingual support

### Layer 3 Tests (pii_layer1_presidio_tests.rs)

**Presidio Integration** (18 tests)
- ✅ PresidioMode configuration (Disabled/SpacyOnly/FullML)
- ✅ Memory overhead validation
- ✅ Accuracy expectations per mode
- ✅ Python availability detection
- ✅ Graceful degradation when unavailable
- ✅ Hybrid detection (Presidio + regex)
- ✅ Confidence score validation
- ✅ Runtime mode switching
- ✅ Entity type detection (15+ types)
- ✅ Python path detection
- ✅ Error handling and malformed input
- ✅ Concurrent request handling
- ✅ Default configuration validation
- ✅ Context enhancement integration
- ✅ Performance benchmarking

**Presidio Modes Tested**:
- Disabled: 0 MB overhead, regex-only
- SpacyOnly: 500 MB overhead, 90% accuracy
- FullML: 2048 MB overhead, 95% accuracy

### Integration Tests (pii_integration_tests.rs)

**Cross-Layer Interactions** (25 tests)
- ✅ 3-layer workflow (full stack detection)
- ✅ Layer fallback mechanisms
- ✅ Layer selection and accuracy validation
- ✅ Cross-layer deduplication
- ✅ Layer status reporting
- ✅ Detection layer switching at runtime
- ✅ Confidence boosting when multiple layers agree
- ✅ Context enhancement across layers
- ✅ Multi-layer performance comparison
- ✅ Error handling and layer isolation
- ✅ Statistics aggregation across layers
- ✅ Redaction with multi-layer detection
- ✅ Anonymization with multi-layer detection
- ✅ Concurrent multi-layer detection
- ✅ Configuration persistence
- ✅ DetectionLayer enum parsing
- ✅ Comprehensive entity coverage test

**Key Integration Scenarios**:
- All 3 layers working together
- Fallback when gline-rs fails
- Fallback when Presidio unavailable
- Configuration changes during runtime
- Concurrent requests across all layers

### Exclusion Tests (pii_exclusions_tests.rs)

**Multi-Regional Exclusions** (20 tests)
- ✅ Legal term exclusions (Supreme Court, First Amendment, etc.)
- ✅ US location exclusions (New York, California, etc.)
- ✅ EU location exclusions (Paris, London, Berlin, etc.)
- ✅ APAC location exclusions (Tokyo, Singapore, etc.)
- ✅ LATAM location exclusions (Mexico City, São Paulo, etc.)
- ✅ Organization exclusions (UN, WHO, EU, etc.)
- ✅ Time term exclusions (Monday, January, etc.)
- ✅ Case-insensitive matching
- ✅ Government agency exclusions
- ✅ Legal phrase exclusions (due process, habeas corpus, etc.)
- ✅ Court name exclusions
- ✅ Over-redaction prevention
- ✅ Context disambiguation (Washington = person vs location)
- ✅ Regional merge validation (8 regions)
- ✅ Partial match handling
- ✅ Abbreviation exclusions (USA, FBI, DOJ, etc.)
- ✅ Real PII preservation (exclusions don't block SSN/email)

**Regional Coverage**:
- EN: United States, UK, Commonwealth
- EU: European Union (GDPR compliance)
- APAC: Asia-Pacific
- LATAM: Latin America
- MENA: Middle East & North Africa
- Africa: Sub-Saharan Africa
- South Asia: India, Pakistan, Bangladesh
- CIS: Commonwealth of Independent States

---

## Performance Benchmarks

### Expected Performance Targets

| Layer | Small Doc (<1MB) | Large Doc (10MB) | Memory Overhead |
|-------|-----------------|------------------|----------------|
| Layer 1 (Regex) | <5ms | 20-50ms | 10 MB |
| Layer 2 (gline-rs) | 20-40ms | 200-400ms | 350 MB |
| Layer 3 (Presidio) | 50-200ms | 500ms-2s | 500-2048 MB |
| All 3 Layers | 30-60ms | 300-600ms | 500-2048 MB |

### Concurrent Request Targets
- 50+ concurrent requests should succeed without errors
- Thread-safe operation across all layers
- No race conditions in configuration updates

---

## Code Coverage Goals

### Overall Target: 90%+

**Critical Paths (Must achieve 95%+)**:
- Core detection logic (detect_pii, detect_with_regex, detect_with_gline)
- Fallback mechanisms
- Configuration management
- Entity type mapping

**Important Paths (Must achieve 90%+)**:
- Exclusion loading and matching
- Context enhancement
- Deduplication and filtering
- Confidence scoring

**Nice-to-Have (Target 85%+)**:
- Error logging
- Performance metrics
- Edge case handling

---

## Test Execution Strategy

### 1. Unit Tests (Isolated Layer Testing)
```bash
# Test individual layers
cargo test pii_layer1_presidio_tests --lib
cargo test pii_layer2_regex_tests --lib
cargo test pii_layer2_gline_tests --lib
```

### 2. Integration Tests (Cross-Layer Testing)
```bash
# Test layer interactions
cargo test pii_integration_tests --lib
cargo test pii_exclusions_tests --lib
```

### 3. Full Test Suite
```bash
# Run all PII tests
cargo test --test pii
```

### 4. Coverage Analysis
```bash
# Generate coverage report
cargo tarpaulin --out html --output-dir coverage/
```

---

## Testing Best Practices

### 1. Test Independence
- Each test should be self-contained
- No shared state between tests
- Use `PIIDetector::new()` for fresh instance

### 2. Async Testing
- All tests use `#[tokio::test]` for async support
- Proper await on detector initialization
- Concurrent test safety validated

### 3. Error Handling
- Test graceful degradation
- Verify fallback mechanisms
- Malformed input should not panic

### 4. Performance Validation
- Use `std::time::Instant` for benchmarks
- Assert reasonable time limits
- Track memory usage trends

### 5. Test Data
- Use realistic legal document examples
- Cover multiple entity types per test
- Include edge cases and corner cases

---

## Known Limitations & Future Tests

### Current Gaps
1. **Fuzzy Exclusion Matching**: Not yet implemented
   - "U.S. Supreme Court" doesn't match "Supreme Court"
   - Future: Add strsim-based fuzzy matching tests

2. **Streaming for Large Files**: Not yet implemented
   - Files loaded entirely into memory
   - Future: Add memory-mapped file tests

3. **Custom Model Fine-Tuning**: Not yet implemented
   - Generic models, not legal-domain specific
   - Future: Add model training/loading tests

4. **GPU Acceleration**: Not available
   - CPU-only execution currently
   - Future: Add CUDA/Metal performance tests

### Future Test Additions
- Cross-validation confidence boosting tests
- Active learning system tests
- Federated model training tests
- Multi-language document tests (Spanish, French, German)
- Real-world legal document corpus tests

---

## Test Maintenance

### When to Update Tests

1. **New Entity Types**: Add corresponding detection tests
2. **New Regions**: Add exclusion tests for new region files
3. **Algorithm Changes**: Update expected confidence scores
4. **Performance Improvements**: Adjust benchmark thresholds

### Regression Testing

- Run full test suite before each release
- Monitor code coverage trends
- Track performance benchmarks over time
- Validate against real-world legal documents

---

## Success Criteria

### Test Suite Passes If:
- ✅ All tests pass on CI/CD
- ✅ Code coverage >= 90%
- ✅ No test flakiness (100% reproducible)
- ✅ Performance benchmarks within targets
- ✅ No memory leaks in concurrent tests
- ✅ Graceful degradation verified
- ✅ Multi-regional exclusions validated

---

## Testing Workflow Integration

### Pre-Commit
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test pii --lib
```

### CI/CD Pipeline
```bash
cargo test --all-features
cargo tarpaulin --out xml
cargo bench --no-run  # Compile benchmarks
```

### Release Validation
```bash
cargo test --release
cargo tarpaulin --release
cargo bench
```

---

## Summary

This comprehensive test suite provides:
- **118+ test cases** covering all 3 layers
- **Multi-regional validation** (8 regions, 3,474+ patterns)
- **Performance benchmarks** (latency and memory)
- **Concurrent operation validation** (thread safety)
- **Error handling verification** (graceful degradation)

The tests ensure BEAR-LLM's PII detection system is production-ready with enterprise-grade accuracy and reliability.
