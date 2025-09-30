# 🐻 BEAR AI LLM - Production-Ready Local AI Assistant

**100% Private AI Assistant for Legal and Professional Use**

![Version](https://img.shields.io/badge/version-1.0.5-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

BEAR AI LLM is a fully production-ready desktop application that runs large language models entirely on your local hardware. Complete privacy, enterprise-grade PII protection, and professional features comparable to Ollama, GPT4All, and jan.ai.

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
- **OS**: Windows 10/11, macOS 11+, Ubuntu 20.04+
- **RAM**: 8GB
- **Storage**: 10GB free space
- **CPU**: 4 cores
- **Python**: 3.8+ (for Presidio)

### Recommended
- **RAM**: 16GB+
- **Storage**: 20GB+ free space
- **CPU**: 8+ cores
- **GPU**: NVIDIA with 4GB+ VRAM
- **Python**: 3.10+

### ⚠️ Important Disk Space Notice
- Base installation: ~500MB
- AI models: 2-8GB per model
- Document embeddings: Variable
- Build dependencies: ~2GB

## 🚀 Installation

### Prerequisites

#### Windows
```bash
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
# Select "Desktop development with C++" workload

# Install Node.js 18+
winget install OpenJS.NodeJS

# Install Rust
winget install Rustlang.Rust.MSVC

# Install Python 3.8+
winget install Python.Python.3.11
```

#### macOS
```bash
# Install Homebrew if not installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install node rust python@3.11
```

#### Linux
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install nodejs npm rustc cargo python3-pip python3-dev

# Fedora
sudo dnf install nodejs npm rust cargo python3-pip python3-devel
```

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/BEAR-LLM.git
cd BEAR-LLM

# Install dependencies
npm install

# Start development server
npm run tauri dev

# Build for production
npm run tauri build
```

## 🎯 First Run Setup

On first launch, BEAR AI will:

1. **Setup Wizard** - Guide you through initial configuration
2. **Install Presidio** - Automatic installation of Microsoft Presidio for PII protection
3. **Download Models** - Recommend and download AI models based on your hardware
4. **Configure Storage** - Set up document database and embeddings

The setup wizard will:
- Detect your hardware capabilities
- Recommend appropriate model sizes
- Install Python dependencies
- Download NER models for PII detection
- Create necessary directories

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
- **Microsoft Presidio**: State-of-the-art NER models
- **OpenPipe Integration**: PII-Redact transformer models
- **Custom Patterns**: Regex-based detection fallback
- **Entity Types**: Names, SSNs, credit cards, emails, phones
- **Automatic Scrubbing**: Real-time PII removal

### Auto-Updates
- **GitHub Releases**: Automatic update checking
- **Minisign**: Cryptographic signature verification
- **User Control**: Accept/defer updates
- **Rollback**: Revert to previous versions
- **Silent Updates**: Background downloads

## 🔧 Configuration

### Environment Variables
Create `.env` in project root:

```env
# Optional: HuggingFace token for gated models
HUGGINGFACE_TOKEN=hf_xxxxxxxxxxxxxxxxxxxx

# Optional: Custom model storage
MODEL_PATH=D:/AI-Models

# Optional: Debug logging
RUST_LOG=debug
RUST_BACKTRACE=1

# Optional: Presidio configuration
PRESIDIO_MODELS_PATH=D:/Presidio-Models
```

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
# macOS: ~/Library/Application Support/BEAR AI LLM/models
# Linux: ~/.local/share/BEAR AI LLM/models
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

#### Presidio Installation Issues
- Ensure Python 3.8+ installed
- Run as administrator (Windows)
- Check pip is updated: `pip install --upgrade pip`

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
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS (Universal)
npm run tauri build -- --target universal-apple-darwin

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

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

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open Pull Request

### Development Guidelines
- Follow Rust best practices
- Use TypeScript strict mode
- Write tests for new features
- Update documentation
- Follow conventional commits

## 📄 License

MIT License - See [LICENSE](LICENSE) file

## 🆘 Support

- **GitHub Issues**: https://github.com/yourusername/BEAR-LLM/issues
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