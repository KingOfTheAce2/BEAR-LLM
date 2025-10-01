# GDPR/AI Act Compliance Tests

Comprehensive test suite for BEAR AI's GDPR and AI Act compliance features.

## Test Coverage

- **Overall Coverage Target**: ≥90%
- **Unit Tests**: 60% of suite
- **Integration Tests**: 30% of suite
- **End-to-End Tests**: 10% of suite

## Quick Start

### Run All Tests
```bash
cd src-tauri
cargo test compliance::tests
```

### Run Specific Test Categories
```bash
# Unit tests only
cargo test compliance::tests::unit

# Integration tests only
cargo test compliance::tests::integration

# E2E tests only
cargo test compliance::tests::e2e

# Security tests only
cargo test compliance::tests::security

# Performance tests only
cargo test compliance::tests::e2e::performance
```

### Run with Output
```bash
cargo test compliance::tests -- --nocapture
```

### Generate Coverage Report
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
# Open coverage/index.html in browser
```

## Test Structure

```
src-tauri/src/compliance/tests/
├── mod.rs                          # Test module configuration
├── fixtures/                       # Mock data and test fixtures
│   └── mod.rs
├── unit/                          # Unit tests (60%)
│   ├── export_engine_tests.rs
│   └── pii_detector_tests.rs
├── integration/                   # Integration tests (30%)
│   └── export_workflow_tests.rs
├── e2e/                          # End-to-end tests (10%)
│   ├── user_rights_tests.rs
│   └── performance_tests.rs
└── security/                     # Security tests
    └── injection_tests.rs
```

## Compliance Requirements Tested

### GDPR Articles

#### Article 15 - Right of Access
- ✅ User can access all personal data
- ✅ Data presented in readable format
- ✅ Export includes all data categories

**Test**: `test_gdpr_article_15_right_of_access()`

#### Article 17 - Right to Erasure
- ✅ User can request data deletion
- ✅ Cascading deletion across tables
- ✅ Deletion is logged in audit trail
- ✅ Data cannot be recovered

**Test**: `test_gdpr_article_17_right_to_erasure()`

#### Article 20 - Data Portability
- ✅ Machine-readable format (JSON)
- ✅ Human-readable formats (Markdown, DOCX, PDF)
- ✅ Data integrity verification (SHA-256)
- ✅ Complete data export

**Test**: `test_gdpr_article_20_right_to_portability()`

#### Article 30 - Records of Processing
- ✅ All data operations logged
- ✅ Audit log immutability
- ✅ Timestamp accuracy
- ✅ Queryable audit trail

**Test**: Audit logging tests throughout

### AI Act Requirements

#### Transparency
- ✅ PII detection disclosed to users
- ✅ Processing purposes documented
- ✅ Data retention policies enforced
- ✅ User notification mechanisms

**Test**: `test_ai_act_transparency()`

## Security Testing

### Injection Prevention
- ✅ SQL injection blocked
- ✅ XSS neutralized
- ✅ Path traversal prevented
- ✅ Command injection blocked

**Tests**: `security/injection_tests.rs`

### Data Protection
- ✅ Encryption validation
- ✅ PII redaction accuracy
- ✅ Secure data deletion
- ✅ Access control

## Performance Benchmarks

| Operation | Benchmark | Test |
|-----------|-----------|------|
| Export 1000 messages | <500ms | ✅ |
| PII detection (100KB) | <1s | ✅ |
| Audit log insertion | <10ms | ✅ |
| Data deletion cascade | <2s | ✅ |
| Encryption (1MB) | <500ms | ✅ |

## Test Data

### Mock Users
- `mock_user_full()` - Complete user with data
- `mock_user_empty()` - Empty user (edge case)
- `mock_user_large()` - Large dataset for performance

### PII Test Cases
- SSN: `123-45-6789`
- Credit Card: `4532-1234-5678-9010` (valid Luhn)
- Email: `test@example.com`
- Phone: `+1 (555) 123-4567`
- Medical Records: `MRN: ABC123456`

### Security Test Payloads
- SQL injection attempts
- XSS payloads
- Path traversal attacks
- Command injection

## CI/CD Integration

### GitHub Actions
Tests run automatically on:
- Every push to `main` or `develop`
- Every pull request
- Nightly at 2 AM UTC

See `.github/workflows/compliance-tests.yml`

### Coverage Requirements
- **Minimum**: 90% overall coverage
- **Critical paths**: 100% coverage
- **Statements**: ≥92%
- **Branches**: ≥85%

### Pre-commit Hooks
Add to `.git/hooks/pre-commit`:
```bash
#!/bin/bash
cd src-tauri
cargo test compliance::tests --quiet
if [ $? -ne 0 ]; then
    echo "❌ Compliance tests failed. Commit aborted."
    exit 1
fi
```

## Test Development Guidelines

### Writing New Tests

1. **Follow the AAA pattern**:
   - **Arrange**: Set up test data
   - **Act**: Execute the function
   - **Assert**: Verify the result

2. **Use descriptive test names**:
   ```rust
   #[test]
   fn test_gdpr_article_17_right_to_erasure() { }
   ```

3. **Test one behavior per test**:
   - Each test should verify a single behavior
   - Use multiple tests for multiple scenarios

4. **Clean up resources**:
   ```rust
   let temp_dir = create_temp_dir();
   // ... test code ...
   cleanup_temp_dir(&temp_dir);
   ```

5. **Use fixtures for consistency**:
   ```rust
   let data = mock_user_full();
   ```

### Test Maintenance

- **Monthly**: Review and update test data
- **Quarterly**: Review compliance requirements
- **Annually**: Full test suite audit

## Debugging Failed Tests

### View test output
```bash
cargo test compliance::tests -- --nocapture
```

### Run specific test
```bash
cargo test test_gdpr_article_17_right_to_erasure -- --nocapture
```

### Run with backtrace
```bash
RUST_BACKTRACE=1 cargo test compliance::tests
```

### Check test in isolation
```bash
cargo test compliance::tests::unit::export_engine_tests::test_export_to_json_basic -- --nocapture
```

## Common Issues

### Issue: Tests fail with "table already exists"
**Solution**: Use in-memory database or cleanup properly
```rust
let conn = create_test_db(); // Creates fresh in-memory DB
```

### Issue: Performance tests fail on CI
**Solution**: Adjust timeouts for slower CI environments or skip with:
```rust
#[test]
#[ignore] // Skip in CI
fn test_performance_benchmark() { }
```

### Issue: Async tests fail
**Solution**: Use `#[tokio::test]` instead of `#[test]`
```rust
#[tokio::test]
async fn test_async_function() { }
```

## Resources

- **Test Strategy**: `/docs/compliance/test-strategy.md`
- **GDPR Requirements**: [GDPR Official Text](https://gdpr-info.eu/)
- **AI Act**: [EU AI Act](https://artificialintelligenceact.eu/)
- **Rust Testing Book**: [The Rust Programming Language - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)

## Contributing

When adding new compliance features:

1. Write tests first (TDD)
2. Ensure 90%+ coverage
3. Document GDPR/AI Act articles addressed
4. Add performance benchmarks
5. Update this README

## Support

For questions or issues:
- Open a GitHub issue
- Tag with `compliance` label
- Include test output and error messages
