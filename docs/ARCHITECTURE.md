# 🏗️ BEAR AI LLM - System Architecture

**Understanding the Infrastructure: LLM vs RAG vs PII Detection**

---

## 📊 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                       BEAR AI APPLICATION                        │
│                     (Tauri Desktop - Rust + React)               │
└─────────────────────────────────────────────────────────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
        ┌───────▼──────┐ ┌─────▼──────┐ ┌─────▼──────────┐
        │ LLM Manager  │ │ RAG Engine │ │ PII Detector   │
        │ (Chat/Gen)   │ │ (Search)   │ │ (Privacy)      │
        └──────────────┘ └────────────┘ └────────────────┘
```

---

## 🤖 Component 1: LLM Manager (Text Generation)

**File:** `src-tauri/src/llm_manager_production.rs`

### Purpose
Handles **text generation, chat, and question answering**. This is the "brain" that thinks and talks.

### Models Used
- **TinyLlama-1.1B** (small, fast, 2GB)
- **Mistral-7B** (balanced, 5GB)
- **Llama2-13B** (high quality, 10GB+)

### Technology Stack
- **Candle Framework** (HuggingFace/Rust)
- **GGUF Format** (quantized models)
- **GPU/CPU Inference** (CUDA or fallback)

### Capabilities
✅ Generate human-like text
✅ Chat and converse
✅ Answer questions
✅ Reasoning and logic
✅ Code generation

❌ Cannot search documents efficiently
❌ Cannot convert text to vectors

### Example Usage
```rust
User: "What is a tort in legal terms?"
LLM: "A tort is a civil wrong that causes harm or loss to another
      person, for which the law provides a remedy..."
```

### Model Selection
Users can download and switch between LLM models from HuggingFace:
- Browse models in the UI
- Download from HuggingFace Hub
- Switch active model at runtime
- Models stored in: `%LOCALAPPDATA%\bear-ai-llm\models\`

---

## 📚 Component 2: RAG Engine (Document Search)

**File:** `src-tauri/src/rag_engine_production.rs`

### Purpose
Handles **document retrieval and semantic search**. This is the "library" that finds relevant information.

### Models Used (Embeddings - NOT LLMs!)
- **BGE-Small-EN-V1.5** (default, 150MB, 384-dim)
- **Legal-BERT** (optional, 440MB, 768-dim, legal-specific)
- **Sentence-T5** (optional, 670MB, best quality)

### Technology Stack
- **FastEmbed Library** (efficient embeddings)
- **Vector Database** (in-memory + SQLite)
- **Cosine Similarity** (semantic matching)

### Capabilities
✅ Convert text to vector embeddings
✅ Semantic search (find similar content)
✅ Document chunking and indexing
✅ Hybrid search (vector + keyword)

❌ Cannot generate text
❌ Cannot chat or answer questions
❌ Cannot reason or understand context

### Example Usage
```rust
// Input: Legal contract text
"The licensee shall indemnify the licensor..."

// Output: Vector embedding (384 numbers)
[0.234, -0.456, 0.678, 0.123, ..., 0.891]

// Used for: Finding similar clauses in other documents
```

### How RAG Works
1. **Indexing:** Document → Split into chunks → Convert to embeddings → Store
2. **Search:** User query → Convert to embedding → Find similar chunks → Return top matches
3. **Generation:** LLM reads retrieved chunks → Generates contextual answer

---

## 🔒 Component 3: PII Detector (Privacy Protection)

**File:** `src-tauri/src/pii_detector_production.rs`

### Purpose
Detects and scrubs personally identifiable information (PII) from documents.

### Technology
- **Built-in Regex** (always available, basic)
- **Microsoft Presidio** (optional, enterprise-grade, Python)
- **OpenPipe PII-Redact** (optional, transformer models)

### Detected Entities
- Names (person, organization)
- Social Security Numbers
- Credit card numbers
- Email addresses
- Phone numbers
- Physical addresses
- Dates of birth
- Medical record numbers

---

## 🔄 How Components Work Together

### Example: User Asks About Uploaded Contract

```
┌─────────────────────────────────────────────────────────────┐
│ User: "What does my contract say about liability limits?"  │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 1: PII Detector                                        │
│ • Scan query for sensitive info (none found)                │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 2: RAG Engine                                          │
│ • Convert query to embedding vector                         │
│ • Search document database for similar chunks               │
│ • Return top 5 relevant contract sections                   │
│ → "Section 8.3: Limitation of Liability..."                 │
│ → "The Company's total liability shall not exceed $10,000"  │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 3: LLM Manager                                         │
│ • Receive retrieved contract sections                       │
│ • Generate human-readable answer                            │
│ → "According to Section 8.3 of your contract, liability    │
│    is limited to $10,000 for any claims arising from..."    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📈 Key Differences: LLM vs Embedding Model

| Feature | LLM (Mistral, Llama) | Embedding Model (BGE) |
|---------|---------------------|----------------------|
| **Purpose** | Text generation | Vector search |
| **Size** | 2-13 GB | 150-670 MB |
| **Output** | Human text | Numeric vectors |
| **Can chat?** | ✅ Yes | ❌ No |
| **Can search docs?** | ❌ No (inefficient) | ✅ Yes (purpose-built) |
| **Training** | Language understanding | Semantic similarity |
| **Speed** | Slower (inference) | Fast (vector math) |
| **Use case** | "Answer this question" | "Find similar content" |

---

## 🎯 Why Two Separate Systems?

### ❌ LLM Alone (No RAG)
**Problem:** LLM doesn't know your documents
```
User: "What's in my contract?"
LLM: "I don't have access to your contract, but generally contracts include..."
```

### ❌ RAG Alone (No LLM)
**Problem:** Can find documents but can't explain them
```
User: "What's in my contract?"
RAG: [Returns raw text chunks with no explanation]
```

### ✅ LLM + RAG (Both Working Together)
**Solution:** Smart assistant that knows your documents
```
User: "What's in my contract?"
RAG: [Finds relevant sections]
LLM: "Your contract includes these key terms: 1) Payment of $5,000..."
```

---

## 🔧 Switching Models

### LLM Model Switching (Already Available)
1. Open BEAR AI
2. Click "Model Selection"
3. Browse HuggingFace models
4. Download and activate

**Models stored:** `%LOCALAPPDATA%\bear-ai-llm\models\`

### RAG Model Switching (Coming in v1.0.13)
1. Open Settings → RAG Configuration
2. Select embedding model:
   - **BGE-Small-EN-V1.5** (default, fast, general-purpose)
   - **Legal-BERT** (legal-specific, better for contracts)
   - **Sentence-T5** (best quality, larger size)
3. Download and switch (requires re-indexing documents)

**Models stored:** `.fastembed_cache\`

---

## 📁 File System Layout

```
%LOCALAPPDATA%\bear-ai-llm\
├── models\                    # LLM models (2-13GB each)
│   ├── TinyLlama-1.1B\
│   ├── Mistral-7B\
│   └── Llama2-13B\
├── embeddings\                # Embedding models (150-670MB)
│   └── .fastembed_cache\
│       └── BAAI__bge-small-en-v1.5\
├── rag_index\                 # Document database
│   ├── documents.db           # SQLite metadata
│   └── embeddings.json        # Vector storage
├── presidio\                  # PII models (optional)
│   └── en_core_web_sm\
└── cache\                     # Temporary files
```

---

## 🚀 Performance Characteristics

### Startup Time
- **Without RAG preload:** <5 seconds
- **With RAG preload:** 10-30 seconds (model loading)
- **Solution:** Download RAG model during setup wizard

### Memory Usage
| Component | RAM Usage | VRAM (GPU) |
|-----------|-----------|------------|
| Base App | 200-300 MB | - |
| RAG Engine | 500-800 MB | Optional |
| LLM (7B) | 4-6 GB | 4-8 GB |
| Total (typical) | 5-7 GB | 4-8 GB |

### Disk Space
- **Base installation:** ~500 MB
- **RAG embeddings:** 150-670 MB (per model)
- **LLM models:** 2-13 GB (per model)
- **Document index:** Variable (depends on uploads)

---

## 🔐 Security & Privacy

### 100% Local Processing
- ✅ All LLM inference on-device
- ✅ All RAG searches local
- ✅ No cloud API calls
- ✅ No telemetry or tracking

### Data Storage
- ✅ Documents encrypted at rest (SQLite)
- ✅ PII automatically detected and flagged
- ✅ Optional Presidio for enterprise-grade protection

### Network Access
- ⚠️ Only for model downloads from HuggingFace
- ⚠️ Optional auto-updates (user controlled)
- ✅ Can operate fully offline after setup

---

## 🛠️ Technology Stack Summary

| Layer | Technology |
|-------|-----------|
| **Frontend** | React + TypeScript + Tailwind CSS |
| **Backend** | Rust (Tauri 2.x) |
| **LLM Runtime** | Candle (HuggingFace) |
| **RAG Engine** | FastEmbed + SQLite |
| **PII Detection** | Regex + Microsoft Presidio (Python) |
| **Vector Math** | Cosine similarity |
| **Database** | SQLite (bundled) |
| **Desktop Framework** | Tauri 2.x |

---

## 📚 Further Reading

- [README.md](../README.md) - User guide and installation
- [CONTRIBUTE.md](../CONTRIBUTE.md) - Contribution guidelines
- [Tauri Documentation](https://tauri.app)
- [Candle Framework](https://github.com/huggingface/candle)
- [FastEmbed](https://github.com/qdrant/fastembed)
- [Microsoft Presidio](https://github.com/microsoft/presidio)

---

**Version:** 1.0.12
**Last Updated:** 2025-09-30
**Author:** Ernst van Gassen
