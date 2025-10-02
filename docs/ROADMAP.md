## 🗺️ Roadmap for BEAR AI LLM

### General

- [ ] Mac and Linux support
- [ ] Voice interface with Whisper
- [ ] RAG with vector databases
- [ ] Plugin system for custom tools
- [ ] Enterprise SSO integration (sovereign, non-big tech only)
- [ ] Collaborative features (still local-only)
- [ ] Transcription service (including PII)

### Full Format Support
- ✅ Add `calamine` crate for complete Excel parsing
- ✅ Implement `docx-rs` for enhanced Word document processing
- ✅ Add `pptx` parsing for PowerPoint presentations

### Advanced Features
- ✅ Add HuggingFace LLM recommendation based on laptop or computer hardware specification
- [ ] Implement Tauri auto-updater with GitHub Releases
- [ ] Add update notification UI components
- [ ] OCR support for scanned documents
- [ ] Image extraction from documents
- [ ] Table structure preservation
- [ ] Advanced metadata extraction

### Enterprise Features
- [ ] Batch document processing
- [ ] Custom PII pattern definitions
- [ ] Document version tracking
- [ ] Automated compliance reporting

  ### Centralized Update Distribution
  - [ ] Configure code signing for secure updates
  - [ ] Build custom update server for enterprise deployments
  - [ ] Add update rollback mechanisms
  - [ ] Implement staged rollout capabilities
  - [ ] Enterprise MDM integration (SCCM, Intune)
  - [ ] Delta update optimization for bandwidth efficiency
  - [ ] Update analytics and reporting dashboard
  - [ ] Silent update mode for corporate environments