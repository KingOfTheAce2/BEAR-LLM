# 🐻 BEAR AI LLM - Production-Ready Local AI Assistant

**100% Private AI Assistant for Legal and Professional Use**

![Version](https://img.shields.io/badge/version-1.0.12-blue)
![License](https://img.shields.io/badge/license-Proprietary-red)
![Platform](https://img.shields.io/badge/platform-Windows-blue)
![Author](https://img.shields.io/badge/author-Ernst%20van%20Gassen-green)

BEAR AI LLM is a fully production-ready desktop application that runs large language models entirely on your local hardware. Complete privacy, enterprise-grade PII protection, and professional features.

## ✨ Key Features

- **🔒 100% Local & Private** - All processing on your hardware, no cloud services
- **🛡️ Microsoft Presidio PII Protection** - Enterprise-grade detection and scrubbing of sensitive information
- **📚 Full Document Processing** - PDF, DOCX, XLSX, CSV, PPTX, MD, JSON support with RAG engine
- **🤖 HuggingFace Integration** - Download and run any compatible model
- **⚡ Hardware Optimization** - Automatic model recommendations based on your GPU/CPU
- **🔄 Auto-Updates** - Seamless updates via GitHub Releases with cryptographic signing
- **🎨 Beautiful UI** - Modern interface with light/dark themes
- **📊 Advanced RAG** - Vector embeddings with FastEmbed for semantic search
- **🚀 Production Ready** - No mocks, no placeholders, full implementations

## 🖥️ System Requirements

### Minimum
- **OS**: Windows 10/11 (64-bit)
- **RAM**: 8GB
- **Storage**: 10GB free space
- **CPU**: 4 cores
- **WebView2**: Automatically installed if missing

### Recommended
- **OS**: Windows 11
- **RAM**: 16GB+
- **Storage**: 20GB+ free space
- **CPU**: 8+ cores
- **GPU**: NVIDIA with 4GB+ VRAM

### Optional (Enhanced Features)
- **Python**: 3.8+ (for Microsoft Presidio PII protection - auto-installed via setup wizard)

### ⚠️ Important Disk Space Notice
- Base installation: ~500MB
- AI models: 2-8GB per model
- Document embeddings: Variable
- Build dependencies: ~2GB

## 🚀 Installation

### For End Users (Pre-built Installer)

1. **Download** the latest release (v1.0.12) from [GitHub Releases](https://github.com/KingOfTheAce2/BEAR-LLM/releases/tag/v1.0.12)
2. **Run** the MSI or NSIS (.exe) installer - both work identically
3. **Launch** BEAR AI - the setup wizard will guide you through first-time configuration
4. **(Optional)** The setup wizard will offer to install Microsoft Presidio for enhanced PII protection

**That's it!** All dependencies including WebView2 are automatically installed.

### For Developers (Building from Source)

#### Prerequisites - Windows 10/11 (64-bit)
```bash
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
# Select "Desktop development with C++" workload

# Install Node.js 18+
winget install OpenJS.NodeJS

# Install Rust (MSVC toolchain)
winget install Rustlang.Rust.MSVC
```

**Note**: Python and Presidio are optional and can be installed via the setup wizard. WebView2 is automatically bundled.

### Quick Start

```bash
# Clone the repository
git clone https://github.com/KingOfTheAce2/BEAR-LLM.git
cd BEAR-LLM

# Install dependencies
npm install

# Start development server
npm run tauri dev

# Build for production
npm run tauri build
```

## 🎯 First Run Setup

On first launch, BEAR AI will guide you through an interactive setup wizard:

1. **Welcome Screen** - Introduction to BEAR AI features
2. **Privacy Protection (Optional)** - Choose whether to install Microsoft Presidio for enhanced PII detection
3. **Model Selection** - Select AI model size based on your hardware (Compact/Balanced/Maximum)
4. **Installation** - Automatic setup of selected components

### What Gets Installed Automatically
- **Required**: Core application files, WebView2 runtime
- **Automatic**: Initial AI models, document database, vector embeddings
- **Optional**: Microsoft Presidio (Python-based PII protection)

**The app works out-of-the-box even without Presidio** - basic PII detection is built-in. Presidio provides enterprise-grade enhancement.

## 📋 Features in Detail

### LLM Model Management
- **Real Downloads**: Actual HuggingFace API integration (no simulations)
- **Progress Tracking**: Real-time download and loading status
- **Model Formats**: GGUF, ONNX, SafeTensors support
- **Smart Selection**: Hardware-based recommendations
- **Model Library**: Pre-configured popular models

### Document Processing
- **RAG Engine**: Production retrieval-augmented generation
- **Vector Search**: FastEmbed for semantic similarity
- **Hybrid Search**: Combines vector and keyword search
- **Persistent Storage**: SQLite for metadata and embeddings
- **Chunking**: Smart document splitting for context

### PII Protection
- **Built-in Detection**: Regex-based pattern matching (always available)
- **Microsoft Presidio (Optional)**: State-of-the-art NER models for enhanced accuracy
- **OpenPipe Integration (Optional)**: PII-Redact transformer models
- **Entity Types**: Names, SSNs, credit cards, emails, phone numbers, addresses
- **Real-time Scrubbing**: Automatic PII removal during document processing
- **Works Out-of-Box**: Basic protection without any additional installations

### Auto-Updates
- **GitHub Releases**: Automatic update checking
- **Minisign**: Cryptographic signature verification
- **User Control**: Accept/defer updates
- **Rollback**: Revert to previous versions
- **Silent Updates**: Background downloads

## 🔧 Configuration

### Environment Variables (Optional)
Create `.env` in project root for advanced configuration:

```env
# Optional: HuggingFace token for gated models
HUGGINGFACE_TOKEN=hf_xxxxxxxxxxxxxxxxxxxx

# Optional: Custom model storage location
MODEL_PATH=D:/AI-Models

# Optional: Debug logging
RUST_LOG=debug
RUST_BACKTRACE=1

# Optional: Custom Presidio path (if installed)
PRESIDIO_MODELS_PATH=D:/Presidio-Models
```

**Most users don't need any environment variables** - the application works with sensible defaults.

### Model Recommendations by Hardware

| Hardware | Recommended Models | Parameters |
|----------|-------------------|------------|
| RTX 4090 (24GB) | Llama-2-13B, CodeLlama-13B | 13B |
| RTX 3080 (10GB) | Mistral-7B, Llama-2-7B | 7B |
| RTX 3060 (6GB) | Phi-2, StableLM-3B | 3B |
| CPU Only | TinyLlama, Phi-1.5 | 1-2B |

## 🛠️ API Reference

### Tauri Commands

```typescript
// Send message to LLM
await invoke('send_message', {
  message: string,
  modelName: string
}): Promise<string>

// Download model
await invoke('download_model_from_huggingface', {
  modelId: string
}): Promise<void>

// Process document
await invoke('process_document', {
  filePath: string,
  fileType: string
}): Promise<ProcessResult>

// Scan for PII
await invoke('scan_for_pii', {
  text: string
}): Promise<PIIResult>

// Get hardware info
await invoke('get_hardware_info'): Promise<HardwareInfo>

// RAG search
await invoke('search_documents', {
  query: string,
  limit: number
}): Promise<SearchResult[]>
```

## 🐛 Troubleshooting

### Common Issues

#### Insufficient Disk Space
```bash
# Clean build artifacts
cd src-tauri && cargo clean

# Remove unused models
# Windows: %LOCALAPPDATA%\BEAR AI LLM\models
```

#### Build Errors (Windows)
```bash
# Ensure Visual Studio Build Tools installed
# Temporarily rename Git's link.exe if conflicts occur
ren "C:\Program Files\Git\usr\bin\link.exe" "link.exe.bak"
```

#### Model Download Failures
- Check internet connection
- Verify HuggingFace is accessible
- Ensure sufficient disk space (2x model size)
- Check firewall settings

#### Optional: Presidio Installation Issues
- Presidio is optional - app works without it
- If installing: Ensure Python 3.8+ is available
- Run setup wizard as administrator (Windows)
- The app will skip Presidio if installation fails and use built-in PII detection

## 📁 Project Structure

```
BEAR-LLM/
├── src/                          # React Frontend
│   ├── components/               # UI Components
│   │   ├── ChatMessage.tsx      # Message display
│   │   ├── ModelSelector.tsx    # Model selection
│   │   ├── SetupWizard.tsx      # First-run setup
│   │   └── UpdateNotification.tsx
│   ├── stores/                  # State Management
│   │   └── appStore.ts          # Zustand store
│   └── utils/                   # Utilities
├── src-tauri/                   # Rust Backend
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── llm_manager_production.rs
│   │   ├── rag_engine_production.rs
│   │   ├── pii_detector_production.rs
│   │   ├── presidio_bridge.rs
│   │   └── commands.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/                      # Static Assets
│   ├── fonts/                   # Inter font files
│   └── images/                  # Logos
└── package.json
```

## 🧪 Testing

```bash
# Frontend tests
npm test

# Rust unit tests
cd src-tauri && cargo test

# Integration tests
npm run test:e2e

# Presidio functionality
npm run test:pii
```

## 📦 Building

### Production Builds

```bash
# Windows (currently supported)
npm run tauri build

# Generates:
# - MSI installer: src-tauri/target/release/bundle/msi/*.msi
# - NSIS installer: src-tauri/target/release/bundle/nsis/*.exe
```

**Note**: macOS and Linux builds are planned for future releases.

### Code Signing

```bash
# Generate signing keys
npx @tauri-apps/cli signer generate -w ~/.tauri/myapp.key

# Sign releases
npx @tauri-apps/cli signer sign --private-key ~/.tauri/myapp.key --file ./target/release/bundle/
```

## 🔐 Security

- **Local Processing**: No data leaves your machine
- **No Telemetry**: Zero tracking or analytics
- **Encrypted Storage**: Models and documents encrypted at rest
- **Signed Binaries**: All releases cryptographically signed
- **PII Protection**: Automatic sensitive data scrubbing
- **Secure Updates**: Signature verification for all updates

## 🤝 Contributing

**BEAR AI is proprietary software with open code for transparency.**

We do not accept external code contributions, but we welcome:
- **Bug Reports**: Submit detailed issues with reproduction steps
- **Feature Requests**: Suggest improvements for legal workflows
- **Security Reports**: Responsible disclosure to security@bearai.com
- **Documentation Feedback**: Request clarifications or improvements

See [CONTRIBUTE.md](CONTRIBUTE.md) for complete details on:
- Why the code is open but not open-source
- Legal and compliance considerations
- Professional support options
- Licensing clarification

## 📄 License

**Proprietary License** - Open code, closed source. See [CONTRIBUTE.md](CONTRIBUTE.md) for details.

This software is proprietary and provided for transparency and security auditing by legal professionals. The source code is publicly viewable but not licensed for redistribution, modification, or commercial use without explicit permission.

## 🆘 Support

- **GitHub Issues**: https://github.com/KingOfTheAce2/BEAR-LLM/issues
- **Documentation**: https://docs.bear-ai.com
- **Discord**: https://discord.gg/bear-ai
- **Email**: support@bear-ai.com

## 🙏 Acknowledgments

- [Microsoft Presidio](https://github.com/microsoft/presidio) - PII detection
- [HuggingFace](https://huggingface.co) - Model hosting
- [Tauri](https://tauri.app) - Application framework
- [Candle](https://github.com/huggingface/candle) - LLM inference
- [FastEmbed](https://github.com/qdrant/fastembed) - Vector embeddings
- [OpenPipe](https://github.com/openpipe/pii-redact) - PII models

## 📊 Comparison

| Feature | BEAR AI | Ollama | GPT4All | jan.ai |
|---------|---------|---------|----------|---------|
| Local Processing | ✅ | ✅ | ✅ | ✅ |
| PII Protection | ✅ Enterprise | ❌ | Basic | ❌ |
| Document RAG | ✅ Full | Basic | Basic | ✅ |
| Auto Updates | ✅ | ✅ | ✅ | ✅ |
| HuggingFace | ✅ | Limited | ✅ | Limited |
| UI Quality | Modern | CLI | Good | Modern |
| Legal Focus | ✅ | ❌ | ❌ | ❌ |

---

**Version 1.0.5** - Production Ready - Enhanced UI, Complete Documentation, Full Implementations.