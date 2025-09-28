# 🐻 BEAR AI - Local LLM Assistant

**100% Private AI Assistant for Legal and Professional Use**

BEAR AI is a desktop application that runs large language models entirely on your local hardware. No internet connection required after initial setup, no data collection, complete privacy.

![BEAR AI Interface](preview-system-aware.html)

## ✨ Key Features

- **🔒 100% Local & Private** - All processing on your hardware, no cloud services
- **🛡️ PII Protection** - Automatic detection and removal of sensitive information
- **🎯 System-Aware** - Intelligent hardware compatibility checking prevents crashes
- **🤖 Agent Capabilities** - Local tool use with MCP (Model Context Protocol)
- **📊 Real-Time Monitoring** - GPU/CPU usage tracking with safety limits
- **🤗 Hugging Face Integration** - Browse and download open-source models
- **⚡ GPU Acceleration** - NVIDIA CUDA, AMD ROCm, and CPU fallback support
- **📱 Dual Interface** - Native desktop app + web browser access

## 🖥️ System Requirements

### Minimum
- **OS:** Windows 10/11 (64-bit)
- **CPU:** 4 cores, 3.0 GHz
- **RAM:** 8 GB
- **Storage:** 20 GB free
- **GPU:** Optional (CPU-only mode available)

### Recommended
- **GPU:** NVIDIA RTX 3060+ (12GB VRAM)
- **RAM:** 32 GB
- **Storage:** 100 GB SSD
- **Models:** 7B-13B parameters run smoothly

### High-End
- **GPU:** NVIDIA RTX 4090 (24GB VRAM)
- **RAM:** 64 GB
- **Can run multiple 30B+ models simultaneously**

## 📦 Installation

### Option 1: Download Installer (Recommended)
1. Go to [Releases](https://github.com/yourusername/BEAR-AI/releases)
2. Download `BEAR-AI-Setup-1.0.0.exe`
3. Run installer and follow setup wizard
4. Launch from Start Menu or Desktop

### Option 2: Build from Source
```bash
# Prerequisites: Node.js 18+, Rust 1.70+
git clone https://github.com/yourusername/BEAR-AI.git
cd BEAR-AI
npm install
npm run tauri build
```

## 🚀 Quick Start

1. **Launch BEAR AI** from Start Menu
2. **System Check** - Automatic hardware detection
3. **Browse Models** - Click "🤗 Browse Models" to see compatible LLMs
4. **Download Model** - Choose one based on your system capabilities
5. **Start Chatting** - Type your question and get private AI responses

### Model Recommendations by System
- **RTX 3060 (8GB):** Llama-2-7B, Mistral-7B, Phi-2
- **RTX 3080 (12GB):** Llama-2-13B, CodeLlama-13B, Mixtral-8x7B
- **RTX 4090 (24GB):** Llama-2-70B, GPT-NeoX-20B, CodeLlama-34B

## 🛡️ Privacy & Security

### No Data Leaves Your Device
- ✅ All AI inference runs locally
- ✅ No telemetry or analytics
- ✅ No internet connection required (after setup)
- ✅ PII automatically detected and removed
- ✅ Sandboxed file operations

### What Gets Monitored (Locally Only)
- GPU/CPU temperature and usage
- Memory consumption
- Model performance metrics
- **None of this data is transmitted anywhere**

## 🔧 Advanced Features

### Agent Capabilities (MCP Tools)
- **File Operations:** Read/write documents (sandboxed)
- **Document Search:** Query your local knowledge base
- **Contract Analysis:** Extract key terms and risks
- **Code Execution:** Run Python/SQL safely
- **Legal Research:** Find precedents and citations

### Model Management
- **Quantization Support:** 4-bit, 8-bit, 16-bit models
- **Context Length:** Up to 32K tokens
- **Multi-Model:** Run multiple models simultaneously
- **Auto-Updates:** Download model updates automatically

## 📊 Performance

### Typical Inference Speeds
| GPU | Model Size | Tokens/Second |
|-----|------------|---------------|
| RTX 3060 | 7B | 25-35 |
| RTX 3080 | 13B | 20-30 |
| RTX 4090 | 30B | 15-25 |
| CPU Only | 7B | 2-5 |

## 🔄 Web Interface

Access via browser at `http://localhost:11434` when desktop app is running:
- Same features as desktop app
- Still 100% local (no internet)
- Good for accessing from other devices on network
- Mobile-friendly responsive design

## 📝 License

This software is licensed under a proprietary license that allows personal and commercial use while protecting intellectual property. See [LICENSE](LICENSE) for full terms.

### Third-Party Components
- Tauri Framework (MIT)
- React (MIT)
- Rust (MIT/Apache-2.0)
- See [THIRD_PARTY_LICENSES.txt](THIRD_PARTY_LICENSES.txt) for complete list

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
```bash
git clone https://github.com/yourusername/BEAR-AI.git
cd BEAR-AI
npm install
npm run tauri dev  # Development mode
```

## 📞 Support

- **Documentation:** Available in-app at `http://localhost:11434/docs`
- **Issues:** [GitHub Issues](https://github.com/yourusername/BEAR-AI/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/BEAR-AI/discussions)

## 🗺️ Roadmap

- [ ] Mac and Linux support
- [ ] Voice interface with Whisper
- [ ] RAG with vector databases
- [ ] Plugin system for custom tools
- [ ] Enterprise SSO integration
- [ ] Collaborative features (still local-only)

## ⚖️ Legal Notice

BEAR AI is designed for legal and professional use. Users are responsible for:
- Compliance with applicable laws and regulations
- Proper handling of confidential information
- Verification of AI-generated content
- Backup of important data

**This software provides privacy tools but users must ensure proper data handling practices.**

---

**🐻 BEAR AI - Your Private AI Assistant**

*No clouds, no tracking, just intelligent assistance on your terms.*