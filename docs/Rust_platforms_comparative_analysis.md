# Rust GUI Desktop Applications for LLM Chat

**The Rust ecosystem offers 26 graphical desktop applications for chatting with Large Language Models**, spanning seven GUI frameworks from web-hybrid approaches like Tauri to pure-Rust native solutions. Tauri dominates with 14 applications, while emerging frameworks like Makepad and established tools like GTK demonstrate Rust's versatility in building modern AI interfaces. Most applications support OpenAI or local models via Ollama, with cross-platform support as standard.

This research identified applications across all major Rust GUI frameworks, revealing a maturing but still-developing ecosystem where **privacy-focused local model support and lightweight desktop experiences** are driving architectural decisions. The landscape splits between web-wrapper convenience and pure-Rust performance, with several applications achieving sub-10MB binary sizes while maintaining full LLM chat functionality.

## Tauri-based applications (Web + Rust backend)

Tauri leads the ecosystem with 14 applications, leveraging web technologies for frontends while maintaining Rust backends for performance and security. These applications range from simple ChatGPT wrappers to sophisticated multi-provider clients with local model support.

### Jan - Comprehensive offline-first LLM platform

**Repository:** https://github.com/janhq/jan  
**License:** AGPL-3.0  
**GUI Stack:** Tauri + React  
**Status:** ⚡ Active development (20k+ stars)

The most feature-complete application in the ecosystem, Jan positions itself as a **100% offline ChatGPT alternative** running entirely on-device. Developed by the menloresearch/jan organization, it supports both local models (Llama, Gemma, Qwen, Mistral via llama.cpp and TensorRT-LLM) and cloud providers (OpenAI, Anthropic, Groq, Mistral). Key differentiators include Model Context Protocol (MCP) integration, custom assistant creation with specialized tasks, and an OpenAI-compatible API server at localhost:1337. System requirements scale with model size: 8GB RAM for 3B parameter models, 16GB for 7B, and 32GB for 13B models. The application demonstrates Tauri's capability for complex desktop applications with extensive local AI infrastructure.

### Kaas - Privacy-focused multi-provider client

**Repository:** https://github.com/0xfrankz/Kaas  
**License:** Not specified  
**GUI Stack:** Tauri + React  
**Status:** ⚡ Active (Listed in awesome-tauri)

Kaas prioritizes **security and privacy with local credential storage** and built-in proxy support to bypass network restrictions. It supports OpenAI, Azure, Anthropic Claude, Ollama for local models, and all OpenAI-compatible providers, with Google Gemini and DeepSeek R1 support in progress. Notable features include prompt templates (Chain-of-Thought, COSTAR framework, custom), simultaneous multi-model support from the same provider, and multilingual support (English, Chinese Simplified/Traditional, Japanese, French, German). The Rust core architecture deliberately limits client privileges for enhanced security, making it ideal for enterprise environments with strict data policies.

### Moly - Makepad showcase with agent support

**Repository:** https://github.com/moxin-org/moly  
**License:** Apache-2.0  
**GUI Stack:** Makepad (pure Rust)  
**Status:** ⚡ Active development (1.3k+ stars)

While built with Makepad rather than Tauri, Moly deserves prominent mention as the **flagship application for Project Robius**, demonstrating pure-Rust GUI capabilities. It integrates with the Moxin ecosystem for AI agent composition via MoFA (Modular Framework for Agents), supports OpenAI-compatible providers through a configurable dashboard, and includes Moly Server for downloading and running open-source LLMs locally. The application features model download management with pause/resume, chat settings for system prompts and parameters, and plans for deep integration with Moxin LLM (truly open-source models complying with Model Openness Framework). With mobile support (iOS/Android) planned, Moly represents the vision for cross-platform Rust-native applications at just 11MB binary size.

### Orion - Lightweight multi-assistant manager

**Repository:** https://github.com/taecontrol/orion  
**License:** MIT  
**GUI Stack:** Tauri + React  
**Status:** ⚡ Active (Listed in awesome-tauri)

At **only 11MB**, Orion excels in creating multiple AI assistants with specific goals and personalities. Each assistant has customizable profiles, unique instructions, and dedicated purposes—perfect for users who need specialized AI agents for different tasks (coding assistant, writing coach, research helper). Chat history stores in local SQLite, ensuring privacy. The application demonstrates Tauri's ability to create incredibly lightweight desktop apps while maintaining full functionality. Available through LemonSqueezy with pay-what-you-want pricing (including $0), Orion balances commercial distribution with accessibility.

### fireside-chat - Pure Rust inference stack

**Repository:** https://github.com/danielclough/fireside-chat  
**License:** Not specified  
**GUI Stack:** Tauri + Leptos (Rust WASM frontend)  
**Status:** ⚡ Active development

Formerly "Candle Chat," this represents the **most Rust-centric Tauri application**, using Leptos WASM for the frontend and HuggingFace Candle for local inference. The pure-Rust stack includes Axum WebSockets for real-time communication, SQLite for conversation persistence (local or remote), and YAML-based configuration for models and inference parameters. Supports quantized models, flash attention, context loading, and both single-user and multi-user chat scenarios. The application demonstrates that Tauri doesn't require JavaScript frameworks—Leptos provides a fully Rust-native frontend experience while maintaining Tauri's cross-platform benefits.

### PersonAi - Character-based AI roleplay

**Repository:** https://github.com/0xAdafang/PersonAi  
**License:** MIT  
**GUI Stack:** Tauri + React + TypeScript + TailwindCSS  
**Status:** ⚡ Active, open source

Designed for **storytelling, roleplay, and character simulation**, PersonAi lets users create AI-powered characters with custom avatars, names, roles, and backstories. The local-first architecture with JSON-based storage ensures complete offline functionality and privacy. Features include dynamic character management, role-based persona system, persistent memory with conversation history per character/persona combination, real-time conversation with typing indicators, and human-friendly timestamps. Developed in Montreal, PersonAi integrates with local LLMs via Ollama or llama.cpp through a Go or Python backend, requiring no external dependencies beyond the local inference engine.

### tauri-llama - Leptos-powered Llama interface

**Repository:** https://github.com/alcolmenar/tauri-llama  
**License:** Not specified  
**GUI Stack:** Tauri + Leptos (Rust WASM) + TailwindCSS  
**Status:** ⚡ Active development

Another pure-Rust frontend implementation using Leptos, focused specifically on **Llama-based local models**. The application demonstrates modern Rust web development with TailwindCSS styling while maintaining the Tauri security model. This project serves as a template for developers wanting to build Tauri applications with Rust throughout the entire stack, avoiding JavaScript/TypeScript dependencies in the frontend.

### DesTalk - ChatGPT clone with planned features

**Repository:** https://github.com/Nuzair46/DesTalk  
**License:** Not specified  
**GUI Stack:** Tauri + React.js  
**Status:** ⚡ Active development with roadmap

A straightforward **ChatGPT clone for desktop** supporting GPT-4 and GPT-3.5 via OpenAI API keys. Cross-platform with a clean interface mimicking ChatGPT's web experience. The roadmap includes ambitious features: Whisper model integration for voice input, image generation support, multiple user management, incognito chat mode, and multi-model support. Available through GitHub releases with simple setup—download, install, enter API key, and start chatting.

### ChatGPT-Tauri - Simple GPT-3.5 desktop client

**Repository:** https://github.com/TESLA2402/ChatGPT-Tauri  
**License:** Not specified  
**GUI Stack:** Tauri + JavaScript  
**Status:** ⚡ Active, accepting contributions

User-friendly desktop application for **GPT-3.5 architecture natural language conversations**. Designed for accessibility across all technical backgrounds with straightforward text input/response chat interface. Built with pnpm, emphasizing simplicity over feature complexity—ideal for users wanting basic ChatGPT functionality without web browser overhead.

### chatgpt-desktop-app-tauri - Menubar-focused quick access

**Repository:** https://github.com/flaviodelgrosso/chatgpt-desktop-app-tauri  
**License:** Not specified  
**GUI Stack:** Tauri + JavaScript  
**Status:** ⚡ Active development

Unofficial **menubar-integrated ChatGPT client** optimized for quick access via global keyboard shortcuts: Cmd+Shift+G (macOS) or Ctrl+Shift+G (Windows/Linux). The menubar design keeps ChatGPT one keystroke away while working in any application. Includes Tauri Updater integration for seamless automatic updates. Perfect for users who want ChatGPT available as a utility tool rather than a full application window.

### chatgpt-desktop (AringoldX) - Minimal web wrapper

**Repository:** https://github.com/AringoldX/chatgpt-desktop  
**License:** Not specified  
**GUI Stack:** Tauri (web wrapper)  
**Status:** ⚡ Active

Extremely lightweight **wrapper for OpenAI ChatGPT website** with menubar/system tray integration. Privacy-focused with no data transfer beyond standard ChatGPT usage. Binary sizes: Windows 2.7MB, macOS 2.1MB, Linux 2.3MB. Represents the minimalist approach—embed the ChatGPT website in a native window for desktop integration without reimplementing chat functionality.

### chat-ai-desktop - Custom UI with API integration

**Repository:** https://github.com/sonnylazuardi/chat-ai-desktop  
**License:** Not specified  
**GUI Stack:** Tauri + chatbot-ui frontend  
**Status:** ⚡ Active

Wraps the popular **chatbot-ui by @mckaywrigley** in a Tauri desktop application with menubar integration. Uses OpenAI API keys rather than web wrapper approach, providing custom UI controls and configuration. Download sizes: Windows 4.11MB, macOS 3.8MB. Credits the chatbot-ui project for frontend while adding Tauri's native desktop capabilities and system tray integration.

### Yack - macOS Spotlight-style overlay

**Repository:** https://github.com/rajatkulkarni95/yack  
**License:** MIT  
**GUI Stack:** Tauri + pnpm frontend  
**Status:** ⚡ Active (Developer focuses on Octarine app now)

**Spotlight-like interface for GPT-3 APIs** exclusive to macOS. Activated via Ctrl+Shift+Space keyboard shortcut, Yack provides a quick overlay for GPT interactions without disrupting workflow. Menu bar icon provides persistent access. The macOS-native feel and Spotlight-style UX make it ideal for Mac users wanting GPT assistance integrated into their operating system. Available at https://www.yack.fyi/

### QuickGPT - Windows lightweight assistant

**Repository:** https://github.com/dubisdev/quickgpt  
**License:** MIT  
**GUI Stack:** Tauri + Web frontend  
**Status:** ⚡ Active (Listed in awesome-tauri)

Windows-exclusive **lightweight AI assistant with ChatGPT integration**. Features code highlighting, keyboard shortcuts (Alt+A to toggle, Esc to clear input), and quick toggle overlay. Settings interface for API key configuration with planned model selection. The Windows-specific focus allows optimization for platform conventions and integration with Windows-specific features.

### tauri-chatgpt - Fast standalone desktop client

**Repository:** https://github.com/litongjava/tauri-chatgpt  
**License:** Not specified (Pull requests welcome)  
**GUI Stack:** Tauri + Web frontend  
**Status:** ⚡ Active, accepting contributions

Standalone desktop application delivering **ChatGPT without browser overhead**. Fast, lightweight, and cross-platform with direct ChatGPT interaction. Cache folder on Windows: `C:\Users\Administrator\AppData\Local\com.litongjava.tauri.chagpt`. Represents the straightforward approach to desktop ChatGPT—native window, minimal features, maximum performance.

## Native Rust GUI applications (iced framework)

Pure-Rust GUI applications using iced, an Elm-inspired framework known for its simplicity and performance. These applications demonstrate native desktop experiences without web technologies.

### icebreaker - Creator-built reference implementation

**Repository:** https://github.com/hecrj/icebreaker  
**License:** Not specified  
**GUI Stack:** iced  
**Status:** ⚡ Active development (332 stars)

Created by **Héctor Ramón, the original author of iced**, serving as both a functional LLM chat application and a reference implementation showcasing the framework's capabilities. Integrates llama.cpp for local inference with HuggingFace model support, providing a pure-Rust desktop experience. Installation via `cargo install --git https://github.com/hecrj/icebreaker.git` requires llama.cpp or Docker. The application features a video demo in the repository and represents best practices for iced application architecture. As a framework creator's showcase, it demonstrates iced's suitability for real-time AI applications with streaming responses.

### ollama-chat-iced (ochat) - Feature-rich Ollama client

**Repository:** https://github.com/CodersCreative/ollama-chat-iced  
**License:** Not specified  
**GUI Stack:** iced  
**Status:** ⚡ Active with continuous updates

Comprehensive **GUI for Ollama AI with modern features**: full markdown support for rich text formatting, voice call integration for dynamic interactions, mic transcription for hands-free input, panel system for multi-tasking, and simultaneous conversations with multiple Ollama models. Download and configure models directly within the application, adjust parameters like temperature, and manage multiple model conversations in parallel. Installation via `cargo install ochat` or building from source. The application demonstrates iced's capability for complex layouts with multiple panels and real-time streaming—essential for responsive LLM chat experiences.

## Native Rust GUI applications (GTK-rs framework)

Applications leveraging GTK for native desktop integration, particularly strong on Linux with mature ecosystem support. GTK-rs brings GNOME's design principles to Rust applications.

### gtk-llm-chat - Multi-provider GTK4 interface

**Repository:** https://github.com/icarito/gtk-llm-chat  
**License:** GPLv3  
**GUI Stack:** GTK4 (gtk4-rs, libadwaita)  
**Status:** ⚡ Active, featured on Hacker News

The most **feature-complete GTK application**, serving as a graphical frontend for the python-llm utility. Supports multiple providers via python-llm integration: OpenAI, Anthropic, Google Gemini, Ollama, and others. Features include independent conversation windows, markdown rendering, sidebar navigation for model/provider selection, model parameter adjustment (temperature, system prompt) per conversation, API key management with banner icons, keyboard shortcuts (F10 sidebar toggle, F2 rename, Escape minimize), conversation management (rename/delete), tray applet for quick access to recent conversations, and dynamic input area with automatic height adjustment. Installation options include Windows installers, Linux AppImage, macOS bundles, or via llm plugin: `pipx install llm; llm install gtk-llm-chat`. Demonstrates GTK4's modern design capabilities with libadwaita integration.

### ollama-rs-gtk - Sleek GTK3 Ollama client

**Repository:** https://github.com/AndresCdo/ollama-rs-gtk  
**License:** MIT  
**GUI Stack:** GTK3 (gtk-rs)  
**Status:** ⚡ Active

Sleek Rust application with **intuitive GTK interface seamlessly integrating Ollama API**. Tested specifically with Ollama Llama 3, providing straightforward text field input with Send button. Configuration via config.toml for API endpoints and model parameters. Requirements: GTK3 version 3.24+ and Ollama service running locally. Cross-platform support across operating systems supporting Rust and GTK. Represents the accessible approach to GTK development—simple interface, clear configuration, reliable Ollama integration.

## Native Rust GUI applications (egui framework)

Immediate-mode GUI applications using egui, popular in the Rust ecosystem for its simplicity and game-development heritage.

### Ellama - Multimodal Ollama interface

**Repository:** https://github.com/zeozeozeo/ellama  
**License:** Unlicense OR MIT OR Apache-2.0 (triple licensed)  
**GUI Stack:** egui  
**Status:** ⚡ Active (Listed in Ollama's official integrations)

Friendly interface for **local or remote Ollama instances with multimodality support**. Create, delete, and edit model settings per-chat with chat history management. Multimodal capabilities enable vision features of any compatible model (e.g., LLaVA) through drag-and-drop, click-to-add, or paste-from-clipboard image support. Optional TTS (text-to-speech) feature requires libspeechd on Linux. Resource efficient with minimal RAM and CPU usage, no subscriptions required—just a local Ollama instance. Installation from releases page or `cargo install --path .` (TTS: `cargo build --features tts`). Video demo available in repository showing real-time interaction.

## Alternative framework applications

Applications using emerging or specialized Rust GUI frameworks, demonstrating the breadth of the ecosystem.

### Floneum - Visual workflow editor (Dioxus)

**Repository:** https://github.com/floneum/floneum  
**License:** Not specified  
**GUI Stack:** Dioxus (rewritten from egui in v0.2)  
**Status:** ⚡ Active (Maintained by Dioxus core team member)

Graph editor for **local AI workflows with zero external dependencies**. While not a traditional chat application, Floneum's visual node interface enables building sophisticated LLM workflows. Rewritten from egui to Dioxus for better flexibility and layout capabilities, demonstrating successful migration path. Features include drag-and-drop visual programming, WebAssembly-based plugin system (write plugins in any WASM-compilable language: Rust, C, Java, Go), sidebar for editing large text blocks, web scraping with Article plugin, RSS feed support, Python integration, browser automation (create, find, click, type, navigate), embedding and vector database for semantic search, and structured generation from LLMs. Supports Llama 2, Mistral, Phi-3 via Kalosm framework, running entirely locally. Related projects include kalosm-chat (Dioxus chat interface) and dioxus-streaming-llm. Available on Dioxus "Made with Dioxus" showcase, with nightly builds for every commit.

### slint-chatbot-demo - Candle inference showcase (Slint)

**Repository:** https://github.com/rustai-solutions/slint-chatbot-demo  
**License:** Not specified  
**GUI Stack:** Slint  
**Status:** Demo/Example project

Demonstration of **Rust + Slint + Candle + OpenChat LLM** integration. Pure Rust implementation showcasing Slint's capabilities for LLM chat applications. Uses openchat_3.5.Q4_K_M.gguf model with local inference via Candle. Installation requires downloading the model via huggingface-cli and tokenizer.json, then `cargo run`. While a demo project, it proves Slint's viability for AI applications and provides a starting template for developers wanting to use Slint's declarative UI approach.

### ChatGPT-rs - Custom native GUI (Framework unclear)

**Repository:** https://github.com/99percentpeople/ChatGPT-rs  
**License:** MIT  
**GUI Stack:** Native/Custom (not explicitly specified)  
**Status:** ⚡ Active

Lightweight **ChatGPT client at ~7MB** with custom native GUI implementation. Features include user-friendly interface, model parameter tuning, conversation history saving, markdown support, syntax highlighting for code snippets, tabbed interface for multiple simultaneous chats, proxy support, system message configuration, and cross-platform support. Pre-built binaries available for Windows, Mac, and Linux. The small binary size and native feel suggest a custom Rust GUI implementation, potentially using direct OS APIs or a minimal framework. Demonstrates that effective LLM clients don't require heavy frameworks.

### chatgpt-gui - GTK4 minimal frontend (gtk4-rs)

**Repository:** https://github.com/teunissenstefan/chatgpt-gui  
**License:** Not specified  
**GUI Stack:** GTK4  
**Status:** Development/Experimental

**GTK4 frontend to ChatGPT completions** in early development. Provides native GUI interface for ChatGPT using modern GTK4 APIs. Build with `cargo build --release`. Planned features include input validation (only digits in certain fields), confirmation dialog on close, preference for "stop" finish_reason, double-click message copying, auto-scroll on new messages, and conversation continuation. Represents the minimalist approach to GTK development—basic functionality first, refinement through iteration.

### opcode/Claudia - Claude Code command center (Tauri 2)

**Repository:** https://github.com/winfunc/opcode  
**License:** AGPL  
**GUI Stack:** Tauri 2 + React + TypeScript  
**Status:** ⚡ Active (Website at claudia.so)

Powerful desktop application that **transforms interaction with Claude Code** (Anthropic's agentic coding tool). Acts as a command center with visual project management, custom AI agent creation with system prompts, interactive Claude Code session management, and secure background agent execution. Features usage tracking and analytics dashboard, cost monitoring and optimization, import configurations from Claude Desktop, multi-agent support with process isolation, permission control (file and network access per agent), local storage (all data stays on machine), no telemetry, and open source transparency. Built with Tauri 2, demonstrating the framework's evolution for complex enterprise applications. Available at https://claudia.so with cross-platform support (macOS, Linux, Windows).

## Framework and provider distribution

The ecosystem shows clear patterns in framework adoption and LLM provider support:

**GUI Framework Distribution:**
- **Tauri:** 14 applications (54%) - Dominant due to web technology familiarity and rapid development
- **iced:** 2 applications (8%) - Growing with framework maturity
- **GTK-rs:** 3 applications (12%) - Strong Linux ecosystem integration
- **egui:** 1 application (4%) - Popular but less represented in LLM space
- **Makepad:** 1 application (4%) - Emerging pure-Rust showcase
- **Dioxus:** 1 application (4%) - Workflow editor rather than traditional chat
- **Slint:** 1 application (4%) - Demo/example project
- **Custom/Native:** 2 applications (8%) - Direct OS integration

**LLM Provider Support:**
- **OpenAI/ChatGPT:** 13 applications - Most common due to API maturity
- **Ollama:** 6 applications - Growing for local model deployment
- **Local models (Candle/llama.cpp):** 5 applications - Privacy-focused use cases
- **Anthropic Claude:** 2 applications - Limited but specialized (Kaas, opcode/Claudia)
- **Multiple providers:** 3 applications - Enterprise and power-user focused (Jan, Kaas, gtk-llm-chat)

**Platform Support:** All applications claim Windows, macOS, and Linux support, except Yack (macOS only) and QuickGPT (Windows only), demonstrating Rust's cross-platform strength.

## Architecture patterns and technical approaches

Three distinct architectural patterns emerge:

**Web Wrappers** embed the ChatGPT website in native windows (chatgpt-desktop, chat-ai-desktop). Minimal at 2-4MB, these provide instant access to latest ChatGPT features without maintenance but lack offline capability or customization.

**API Integrations** connect to LLM provider APIs with custom UIs (ChatGPT-rs, DesTalk, Orion, Kaas). These offer middle-ground flexibility—customizable interfaces, local data storage, and API control—while requiring API keys and internet connectivity. Binary sizes range from 7-15MB.

**Local Inference Engines** run models entirely on-device (icebreaker, fireside-chat, Ellama, PersonAi). Maximum privacy and offline capability come at the cost of larger downloads (models are several GB), higher system requirements, and more complex setup. These applications demonstrate Rust's performance capabilities for AI inference.

The pure-Rust approaches (Moly with Makepad, fireside-chat and tauri-llama with Leptos) show the ecosystem moving beyond JavaScript dependencies, though Tauri+React remains dominant for rapid development.

## Maturity assessment and development status

The ecosystem remains **in active development but pre-production** for most applications. Only Jan, Moly, and gtk-llm-chat show production-ready polish with comprehensive documentation, regular releases, and large user bases. Most projects are maintained by individual developers or small teams, leading to varying documentation quality and update frequency.

**Missing gaps include:** Enterprise features (SSO, audit logs, compliance), extensive multi-modal support (only Ellama and planned in some Tauri apps), fine-tuned model support beyond major providers, and collaborative features (team chat, sharing). The ecosystem would benefit from more applications supporting Anthropic Claude, Google Gemini, and newer providers like Mistral AI.

**Licensing varies widely**, with many projects not specifying licenses—a concern for commercial adoption. Specified licenses include MIT (more permissive), AGPL (copyleft), and GPLv3 (strong copyleft), affecting integration possibilities.

## Installation and deployment

Most applications provide **multiple installation methods**: pre-built binaries (Windows .msi/.exe, macOS .dmg/.app, Linux .deb/.rpm/AppImage), Cargo installation (`cargo install` from crates.io or git), and source builds (`cargo build --release`). 

The Tauri applications demonstrate Rust's ability to produce small binaries: QuickGPT, Yack, and chatgpt-desktop range from 2-11MB, while feature-rich applications like Jan and Moly distribute as full packages with bundled dependencies. The iced and GTK applications typically require system libraries (GTK3/4), affecting portability but enabling deep OS integration.

**System requirements vary significantly:** Simple API clients run on minimal hardware, while local inference applications (Jan, icebreaker, Ellama) require 8-32GB RAM depending on model size and may benefit from GPU acceleration.

## Future directions and emerging patterns

The ecosystem trends toward **local-first architectures** with privacy preservation, driven by concerns about data sovereignty and API costs. Applications like Jan, PersonAi, and Ellama demonstrate viable local LLM deployment with acceptable performance on consumer hardware.

**Multi-agent systems** emerge as a pattern, with Moly's MoFA integration and opcode/Claudia's multi-agent support showing interest in specialized AI assistants beyond single-model chat. This aligns with industry movement toward agentic AI workflows.

The **pure-Rust GUI movement** (Makepad, Leptos frontends, iced) suggests growing dissatisfaction with web technology overhead, even in Tauri applications. As these frameworks mature, expect more applications abandoning JavaScript frontends entirely.

**Missing major players:** No applications yet support cutting-edge providers like Perplexity, together.ai, or Replicate. The ecosystem lacks applications optimized for specific use cases like code generation (beyond opcode/Claudia), document analysis, or creative writing—opportunities for specialized tools.

## Conclusion

Rust delivers 26 graphical LLM chat applications spanning beginner-friendly web wrappers to sophisticated local inference engines. While Tauri dominates current development, emerging pure-Rust frameworks demonstrate the language's potential for native desktop AI applications without web technology dependencies. The ecosystem remains young but growing rapidly, with clear trajectories toward local-first privacy, multi-agent architectures, and specialized use cases beyond general chat. For developers choosing a framework, Tauri offers maturity and rapid development; iced provides pure-Rust simplicity; GTK-rs delivers deep Linux integration; and Makepad represents the future of cross-platform Rust native applications. For users, the choice depends on priorities: privacy (local models with Jan/Ellama), convenience (web wrappers), or customization (API integrations with custom UIs).