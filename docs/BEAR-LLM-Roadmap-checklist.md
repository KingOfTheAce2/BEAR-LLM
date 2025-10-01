# 🗺️ Roadmap for BEAR-LLM (Legal AI)

## Phase 1 (Months 1--6): Foundation of Trust & Utility (MVP)

```{=html}
<details>
```
```{=html}
<summary>
```
General
```{=html}
</summary>
```
-   [ ] **Cross-platform desktop application**
    -   [ ] Support Windows 10+\
    -   [ ] Support macOS (Intel + Apple Silicon)\
    -   [ ] Support Linux (Ubuntu, Fedora as primary targets)\
    -   [ ] Automated build pipeline (CI/CD) for multi-OS binaries
        (GitHub Actions)\
    -   [ ] Package installers (.exe, .dmg, .AppImage/.deb)
-   [ ] **Rust backend**
    -   [ ] File ingestion (PDF, DOCX, TXT initially)\
    -   [ ] Text extraction pipeline with error handling\
    -   [ ] Indexing engine with Tantivy\
    -   [ ] Internal API interface to TypeScript front-end
-   [ ] **Encryption & Security**
    -   [ ] AES-256 encryption for data at rest\
    -   [ ] Encrypted vault file system per project\
    -   [ ] Secure password-based project access\
    -   [ ] Local key storage (OS keychain integration)
-   [ ] **Local RAG pipeline**
    -   [ ] Integration of quantized open-source LLM (Llama 3 /
        Mistral)\
    -   [ ] CPU inference baseline with optional GPU acceleration
        (CUDA/Metal)\
    -   [ ] Embedding generation (sentence-transformers or Rust-native
        equivalent)\
    -   [ ] Local vector DB for embedding storage
-   [ ] **SOC 2 Type II documentation (start process)**
    -   [ ] Define scope of SOC 2 audit (security, confidentiality,
        availability)\
    -   [ ] Draft security policies (access control, incident response,
        change management)\
    -   [ ] Document data encryption approach\
    -   [ ] Document local-first architecture and zero-knowledge design\
    -   [ ] Start evidence collection (logs, access controls, test
        results)\
    -   [ ] Define roles and responsibilities (who approves changes, who
        monitors logs)\
    -   [ ] Vendor management policy (covering Stripe, CI/CD,
        third-party libraries)\
    -   [ ] Draft incident response playbook (steps, roles, escalation
        matrix)

```{=html}
</details>
```
```{=html}
<details>
```
```{=html}
<summary>
```
Core Features
```{=html}
</summary>
```
-   [ ] Encrypted project vaults (password-protected workspaces)\
-   [ ] Local document ingestion (PDF, DOCX, TXT)\
-   [ ] Q&A "Chat with Document/Project" interface (local inference)\
-   [ ] Verifiable source citations with clickable references\
-   [x] PII scrubbing with Microsoft Presidio
    -   [ ] Configure Presidio with default patterns (SSN, email, phone,
        IBAN, credit card)\
    -   [ ] Add custom legal-sensitive patterns (contract numbers, case
        IDs, client IDs)\
    -   [ ] Ensure scrubbing pipeline runs **before ingestion**\
    -   [ ] Option for user to toggle PII scrubbing per project

```{=html}
</details>
```
```{=html}
<details>
```
```{=html}
<summary>
```
Monetization
```{=html}
</summary>
```
-   [ ] Tiered subscription model (Pro, Team)\
-   [ ] Stripe billing integration\
-   [ ] Fully featured free trial (14-day)

```{=html}
</details>
```

------------------------------------------------------------------------

## Phase 2 (Months 7--18): Differentiation & Intelligence

```{=html}
<details>
```
```{=html}
<summary>
```
Architecture & Compliance
```{=html}
</summary>
```
-   [ ] GraphRAG engine
    -   [ ] Entity extraction (parties, dates, jurisdictions, defined
        terms)\
    -   [ ] Relationship storage in embedded graph DB\
    -   [ ] Hybrid retrieval pipeline (vector search + graph traversal)
-   [ ] Agentic workflow engine (v1)
    -   [ ] Chain multiple tasks (classify → extract → compare →
        summarize)\
    -   [ ] Define task specification format (YAML/JSON)\
    -   [ ] Local orchestration layer
-   [ ] Complete SOC 2 Type II & ISO 27001 certification
    -   [ ] Perform gap analysis\
    -   [ ] Engage external auditor\
    -   [ ] Collect 6 months of control evidence\
    -   [ ] Prepare audit-ready documentation

```{=html}
</details>
```
```{=html}
<details>
```
```{=html}
<summary>
```
Features
```{=html}
</summary>
```
-   [ ] Visual Knowledge Graph Explorer (interactive UI for clauses,
    parties, links)\
-   [ ] Cross-document analysis (comparisons, clause consistency)\
-   [ ] Automated Playbooks (checklists for NDAs, MSAs, compliance
    standards)\
-   [ ] Advanced redlining assistant (auto-suggest edits for flagged
    clauses)

```{=html}
</details>
```
```{=html}
<details>
```
```{=html}
<summary>
```
Monetization
```{=html}
</summary>
```
-   [ ] Enterprise subscription tier (Playbooks, Redlining, admin
    controls)\
-   [ ] Seat-based pricing for Team and Enterprise tiers

```{=html}
</details>
```

------------------------------------------------------------------------

## Phase 3 (Months 19--36): Market Leadership & Ecosystem Expansion

```{=html}
<details>
```
```{=html}
<summary>
```
Architecture & R&D
```{=html}
</summary>
```
-   [ ] Mature agentic workflow engine → multi-agent orchestration
    platform\
-   [ ] Secure integration framework for third-party tools
    -   [ ] Clio\
    -   [ ] NetDocuments\
    -   [ ] e-discovery platforms\
-   [ ] Research into privacy-preserving ML (federated learning, secure
    aggregation)

```{=html}
</details>
```
```{=html}
<details>
```
```{=html}
<summary>
```
Features
```{=html}
</summary>
```
-   [ ] Custom Agent Builder (no-code/low-code interface for legal
    workflows)\
-   [ ] Predictive analytics (local-only, opt-in)\
-   [ ] Integration marketplace for legal software

```{=html}
</details>
```
```{=html}
<details>
```
```{=html}
<summary>
```
Monetization
```{=html}
</summary>
```
-   [ ] Usage-based pricing (credits for advanced workflows)\
-   [ ] Secure API access for enterprise deployments\
-   [ ] Professional services (custom deployment, training, support)

```{=html}
</details>
```
