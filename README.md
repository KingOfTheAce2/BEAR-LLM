# 🐻 BEAR AI LLM - Local AI LLM Assistant

**100% Private AI Assistant for Legal and Professional Use**

BEAR AI LLM is a desktop application that runs large language models entirely on your local hardware. No internet connection required after initial setup, no data collection, complete privacy.

## ✨ Key Features

- **🔒 100% Local & Private** - All processing on your hardware, no cloud services
- **🛡️ Advanced PII Protection** - Real-time detection and scrubbing of sensitive information
- **🧠 Agentic RAG System** - Intelligent document search with reasoning and query rewriting
- **📚 Smart Document Processing** - 14+ file formats with automatic indexing
- **🎯 System-Aware** - Intelligent hardware compatibility checking prevents crashes
- **🤖 Agent Capabilities** - Local tool use with MCP (Model Context Protocol)
- **💼 Corporate-Optimized Models** - Pre-selected models for business laptops
- **🔍 HuggingFace Integration** - Browse 300,000+ models with smart filtering
- **📊 Real-Time Monitoring** - GPU/CPU usage tracking with safety limits
- **⚡ GPU Acceleration** - NVIDIA CUDA, AMD ROCm, and CPU fallback support
- **📱 Case-Based Interface** - Professional workflow with conversation management
- **🛡️ System Guardrails** - Crash prevention and resource management

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
1. Go to [Releases](https://github.com/KingOfTheAce2/BEAR-LLM/releases)
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

1. **Launch BEAR AI LLM** from Start Menu
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

## 🧠 Agentic RAG & Document Intelligence

### Advanced RAG Capabilities
- **🤖 AgenticRAG Mode** - Multi-step reasoning with query rewriting and result reranking
- **📄 Smart Document Processing** - Automatic text extraction from 14+ file formats
- **🔍 Semantic Search** - Vector embeddings with relevance scoring
- **🧩 Intelligent Chunking** - Context-aware document segmentation
- **📝 Answer Generation** - Cited responses with confidence scoring
- **🔄 Query Enhancement** - Automatic question expansion and refinement

### Document Format Support
| Category | Formats | PII Detection | RAG Indexing |
|----------|---------|---------------|--------------|
| **Text** | TXT, MD, RTF | ✅ | ✅ |
| **Office** | PDF, DOCX, DOC | ✅ | ✅ |
| **Data** | JSON, CSV, XML, HTML | ✅ | ✅ |
| **Spreadsheets** | XLSX, XLS | ✅ | ✅ |
| **Presentations** | PPTX, PPT | ✅ | ✅ |

### AgenticRAG Workflow
1. **Query Analysis** - Intent detection and complexity assessment
2. **Query Rewriting** - Multiple formulation strategies for better retrieval
3. **Semantic Retrieval** - Vector search across indexed documents
4. **Context Ranking** - Relevance scoring and result reordering
5. **Answer Synthesis** - Multi-source reasoning with citations
6. **Quality Assessment** - Confidence scoring and result validation

## 🔧 Advanced Features

### Case-Based Professional Interface
- **📋 Case Management** - Organize conversations by legal cases or projects
- **🔒 PII-Safe Conversations** - Real-time sensitive data detection and blocking
- **📖 Conversation History** - Persistent storage with search capabilities
- **🎯 Context Awareness** - Maintains case context across sessions
- **👥 Multi-Case Support** - Switch between different matters seamlessly

### Corporate Model Selection
- **🏢 Pre-Screened Models** - Optimized for HP/Lenovo/Dell corporate laptops
- **📊 Performance Metrics** - Words/second alongside tokens/second
- **💾 Resource Monitoring** - RAM, disk, and GPU usage tracking
- **⚖️ Size Categories** - Lightweight (<1GB), Medium (1-5GB), Large (5GB+)
- **🔧 Auto-Configuration** - Automatic settings based on hardware detection

### Agent Capabilities (MCP Tools)
- **📁 File Operations** - Read/write documents (sandboxed)
- **🔍 Document Search** - Query your local knowledge base with AgenticRAG
- **📄 Contract Analysis** - Extract key terms, risks, and obligations
- **💻 Code Execution** - Run Python/SQL safely in isolated environment
- **⚖️ Legal Research** - Find precedents, citations, and case law
- **🛡️ PII Detection** - Advanced pattern matching for sensitive data
- **📊 Data Analytics** - Statistical analysis and reporting tools

### Model Management
- **📦 Quantization Support** - 4-bit, 8-bit, 16-bit models for efficiency
- **📏 Context Length** - Up to 32K tokens for large documents
- **🔄 Multi-Model** - Run multiple specialized models simultaneously
- **🔄 Auto-Updates** - Smart model updates with rollback capability
- **🌐 HuggingFace Browser** - Search and filter 300,000+ available models
- **⚡ Performance Optimization** - Dynamic model switching based on task complexity

## 📊 Performance

### Typical Inference Speeds
| GPU | Model Size | Tokens/Second | Words/Second | Use Case |
|-----|------------|---------------|--------------|----------|
| RTX 3060 | 7B | 25-35 | 18-26 | Legal research, contract review |
| RTX 3080 | 13B | 20-30 | 15-22 | Complex reasoning, document analysis |
| RTX 4090 | 30B | 15-25 | 11-18 | Advanced legal AI, multi-document synthesis |
| CPU Only | 7B | 2-5 | 1.5-3.7 | Basic document processing |

### Corporate Laptop Performance
| Laptop Category | Recommended Model | RAM Usage | Performance |
|-----------------|-------------------|-----------|-------------|
| **Budget Business** | TinyLlama 1.1B | 3-4GB | 12-18 words/sec |
| **Standard Corporate** | DialoGPT Medium | 6-8GB | 18-26 words/sec |
| **High-Performance** | Phi-2 2.7B | 8-12GB | 15-22 words/sec |

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

## ⚖️ Legal Notice

BEAR AI LLM is designed for legal and professional use. Users are responsible for:
- Compliance with applicable laws and regulations
- Proper handling of confidential information
- Verification of AI-generated content
- Backup of important data

**This software provides privacy tools but users must ensure proper data handling practices.**

---

**🐻 BEAR AI LLM - Your Private AI Assistant**

*No clouds, no tracking, just intelligent assistance on your terms.*