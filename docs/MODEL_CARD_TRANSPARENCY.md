# HuggingFace Model Card Transparency Implementation

## Overview

This implementation provides automatic fetching and display of HuggingFace model cards to ensure AI transparency and EU AI Act compliance. When users load GGUF models, the system automatically retrieves and displays model capabilities, limitations, and ethical considerations.

## Key Features

### 1. **Automatic Model Card Fetching** (`model_card_fetcher.rs`)
- Fetches model metadata from HuggingFace Hub API
- Downloads and caches README.md files containing model cards
- Implements local caching with 7-day TTL to minimize API calls
- Gracefully handles offline mode with stale cache fallback
- HTTP client with 10-second timeout for responsiveness

### 2. **Model Identifier Mapping** (`model_registry.rs`)
- Maps GGUF filenames to HuggingFace model IDs
- Pre-configured patterns for popular models:
  - Llama 2/3 (meta-llama)
  - Mistral/Mixtral (mistralai)
  - Phi (microsoft)
  - Gemma (google)
  - And 10+ more model families
- Supports custom user-defined mappings
- Handles filename variations (underscores, hyphens, case differences)

### 3. **Model Card Parsing** (`model_card_parser.rs`)
- Extracts structured data from markdown model cards:
  - Model description
  - Intended use cases
  - Known limitations
  - Biases and fairness considerations
  - Training data information
  - License information
  - Research paper URLs
  - Safety warnings
- Uses regex patterns to handle different markdown formats
- Robust parsing for inconsistent model card structures

### 4. **Disclaimer Generation** (`disclaimer_generator.rs`)
- Generates human-readable disclaimers from model cards
- Highlights key warnings and limitations prominently
- Lists recommended and NOT recommended use cases
- Determines if user acknowledgment is required
- Formats disclaimers for both full display and inline use
- Contextual warnings based on model characteristics

### 5. **Generic Fallback Disclaimers** (`generic_disclaimer.rs`)
- Provides disclaimers when model cards unavailable
- Multiple disclaimer types:
  - **Unknown Model**: For unrecognized models
  - **Offline Mode**: When network unavailable
  - **General AI**: Standard AI limitations
  - **High-Risk**: Critical warning for sensitive applications
  - **EU AI Act**: Regulatory compliance notice
- Severity levels: Info, Warning, Critical
- Color-coded display based on severity

### 6. **UI Component** (`ModelInfoPanel.tsx`)
- React component for displaying model information
- Collapsible sections for:
  - Warnings (expanded by default)
  - Intended use cases
  - Limitations
  - Biases
  - Not recommended uses
- User acknowledgment checkbox for critical warnings
- Persists acknowledgment in localStorage
- Links to full model card on HuggingFace
- Responsive design with dark mode support

### 7. **Tauri Commands** (`model_transparency.rs`)
- `get_model_info`: Fetch complete model information
- `add_model_mapping`: Add custom GGUF→HuggingFace mapping
- `remove_model_mapping`: Remove custom mapping
- `get_model_mappings`: List all mappings
- `clear_model_cache`: Clear cache for specific model
- `clear_all_model_cache`: Clear all cached model cards
- `get_general_disclaimer`: Get standard AI disclaimer
- `get_ai_act_disclaimer`: Get EU AI Act compliance notice
- `get_high_risk_disclaimer`: Get critical warning
- `format_disclaimer_display`: Format for full display
- `format_generic_disclaimer_display`: Format generic disclaimer

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│                   (ModelInfoPanel.tsx)                       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ Tauri Commands
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                 Model Transparency Service                   │
│              (ModelTransparencyState)                        │
└────┬─────────────────────────────────────────────┬──────────┘
     │                                              │
     │                                              │
     ▼                                              ▼
┌─────────────────────┐                  ┌──────────────────────┐
│  Model Registry     │                  │  Model Card Fetcher  │
│  (GGUF → HF ID)     │                  │  (API + Cache)       │
└─────────────────────┘                  └──────────┬───────────┘
                                                     │
                                                     ▼
                                         ┌──────────────────────┐
                                         │  Model Card Parser   │
                                         │  (Markdown → Data)   │
                                         └──────────┬───────────┘
                                                     │
                                                     ▼
                                         ┌──────────────────────┐
                                         │ Disclaimer Generator │
                                         │ (Data → User Notice) │
                                         └──────────────────────┘
```

## AI Act Compliance

### Article 13 (Transparency Requirements)
✅ Users are informed when interacting with AI systems
✅ Model capabilities and limitations are clearly disclosed
✅ Known biases and fairness considerations are highlighted
✅ Training data information is provided when available

### Article 52 (Information Obligations)
✅ Clear identification of AI-generated interactions
✅ Transparency about model purpose and intended use
✅ Warning about not recommended use cases
✅ User acknowledgment for high-risk applications

## Usage Example

### Backend (Rust)
```rust
use bear_ai_llm::ai_transparency::{
    ModelCardFetcher, ModelRegistry, ModelCardParser,
    DisclaimerGenerator, GenericDisclaimerGenerator
};

// Get model information
let info = get_model_info("llama-2-7b-chat.Q4_K_M.gguf", state).await?;

// info contains:
// - filename: "llama-2-7b-chat.Q4_K_M.gguf"
// - display_name: "Llama 2 7b Chat"
// - model_id: Some("meta-llama/Llama-2-7b-chat-hf")
// - disclaimer: ModelDisclaimer { ... }
```

### Frontend (TypeScript)
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import ModelInfoPanel from '@/components/models/ModelInfoPanel';

// Display model information panel
<ModelInfoPanel
  modelFilename="llama-2-7b-chat.Q4_K_M.gguf"
  onAcknowledge={() => console.log('User acknowledged warnings')}
/>

// Or fetch programmatically
const info = await invoke('get_model_info', {
  filename: 'llama-2-7b-chat.Q4_K_M.gguf'
});
```

## File Structure

```
src-tauri/src/ai_transparency/
├── mod.rs                      # Module exports
├── model_card_fetcher.rs       # HuggingFace API client
├── model_registry.rs           # Filename → Model ID mapping
├── model_card_parser.rs        # Markdown parsing
├── disclaimer_generator.rs     # Model-specific disclaimers
└── generic_disclaimer.rs       # Fallback disclaimers

src-tauri/src/commands/
└── model_transparency.rs       # Tauri command handlers

src/components/models/
└── ModelInfoPanel.tsx          # React UI component

tests/ai_transparency/
├── mod.rs
├── model_card_tests.rs         # Parser & disclaimer tests
└── model_registry_tests.rs     # Registry mapping tests

docs/
└── MODEL_CARD_TRANSPARENCY.md  # This file
```

## Dependencies

```toml
reqwest = { version = "0.12", features = ["json"] }  # HTTP client
pulldown-cmark = "0.12"  # Markdown parser
serde_json = "1.0"       # JSON serialization
regex = "1"              # Pattern matching
```

## Caching Strategy

- **Cache Location**: `~/.local/share/bear-ai/model_cards/`
- **Cache Format**: JSON files named `{model-id-sanitized}.json`
- **Cache TTL**: 7 days
- **Stale Cache**: Used as fallback if API fails
- **Cache Management**: Commands to clear individual or all caches

## Error Handling

1. **Model ID Not Found**: Shows generic unknown model disclaimer
2. **Network Failure**: Uses stale cache or offline disclaimer
3. **Parse Errors**: Returns basic model info with generic disclaimer
4. **Invalid Model Cards**: Extracts what's available, fills gaps with defaults

## Testing

```bash
# Run all transparency tests
cd src-tauri
cargo test ai_transparency

# Run specific test modules
cargo test model_card_tests
cargo test model_registry_tests
```

## Future Enhancements

1. **Multi-language Support**: Translate disclaimers based on user locale
2. **Model Card Versioning**: Track changes to model cards over time
3. **Custom Disclaimer Templates**: Allow organizations to customize warnings
4. **Automated Risk Assessment**: Analyze model cards to determine risk level
5. **Model Comparison**: Side-by-side comparison of multiple models
6. **Offline Model Cards**: Bundle common model cards with application
7. **Model Card Search**: Full-text search across cached model cards

## Maintenance

- **Update Model Mappings**: Add new models as they become popular
- **Cache Cleanup**: Automatic cleanup of expired cache entries
- **API Monitoring**: Track HuggingFace API availability and errors
- **User Feedback**: Collect feedback on disclaimer clarity and usefulness

## References

- [EU AI Act - Article 13](https://artificialintelligenceact.eu/article/13/)
- [EU AI Act - Article 52](https://artificialintelligenceact.eu/article/52/)
- [HuggingFace Hub API](https://huggingface.co/docs/hub/api)
- [Model Card Toolkit](https://github.com/tensorflow/model-card-toolkit)

---

**Implementation Date**: October 2, 2025
**AI Act Compliance**: Article 13, Article 52
**Status**: ✅ Complete and Integrated
