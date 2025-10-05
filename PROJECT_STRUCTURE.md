# 📁 BEAR-LLM Project Structure

Welcome to the BEAR-LLM project! This document is your guide to understanding the project's structure, even if you don't have a technical background. We'll explain what the different folders and files are for, so you can feel confident navigating the repository.

## 🌳 High-Level Overview

The project is a desktop application built with a modern web technology stack. It's divided into two main parts:

1.  **Frontend (User Interface):** This is what you see and interact with—the buttons, chat windows, and settings. It's written in **TypeScript** using the **React** library, a popular choice for building interactive user interfaces.
2.  **Backend (Core Logic):** This is the "engine" of the application. It runs behind the scenes, handling tasks like loading the AI models, processing data, and managing files. It's written in **Rust**, a language known for its performance and safety.

The two parts communicate with each other through a bridge called **Tauri**, which allows us to build a desktop app using web technologies for the frontend.

Think of it like a car: the frontend is the dashboard, steering wheel, and seats, while the backend is the engine, transmission, and wheels.

```
BEAR-LLM/
├── src/                            # React Frontend
│   ├── components/                 # UI Components
│   │   ├── ChatMessage.tsx         # Message display
│   │   ├── ModelSelector.tsx       # Model selection
│   │   ├── SetupWizard.tsx         # First-run setup
│   │   └── UpdateNotification.tsx
│   ├── stores/                     # State Management
│   │   └── appStore.ts             # Zustand store
│   └── utils/                      # Utilities
├── src-tauri/                      # Rust Backend
│   ├── src/
│   │   ├── main.rs                 # Entry point
│   │   ├── llm_manager_production.rs
│   │   ├── rag_engine_production.rs
│   │   ├── pii_detector_production.rs
│   │   ├── presidio_bridge.rs
│   │   └── commands.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/                         # Static Assets
│   ├── fonts/                      # Inter font files
│   └── images/                     # Logos
└── package.json
```

---

## 📁 Root Directory

Here are the most important files and folders in the main project directory:

| File/Folder | Description |
| :--- | :--- |
| **`src/`** | Contains all the code for the user interface (the "frontend"). |
| **`src-tauri/`** | Contains all the code for the application's core logic (the "backend"). |
| **`public/`** | Holds static files that are directly used by the frontend, like images and fonts. |
| **`docs/`** | A folder containing all project documentation. If you want to understand a specific feature or decision, this is the place to look. |
| **`tests/`** | Contains automated tests that ensure the application works as expected and help prevent bugs. |
| **`package.json`** | A standard file in modern web projects. It lists the project's dependencies (the external libraries it uses) and defines helpful scripts (e.g., for starting the app). |
| **`README.md`** | The front page of the project. It gives a general overview and often contains setup instructions. |
| **`CONTRIBUTE.md`** | Guidelines for contributing to the project. |
| **`LICENSE`** | The project's software license. |

---

## 🎨 Frontend (`src/`)

This folder contains the code that creates the user interface. It's written in TypeScript and React.

| File/Folder | Description |
| :--- | :--- |
| **`main.tsx`** | The entry point of the React application. It's the first file that gets loaded. |
| **`App.tsx`** | The main component of the application. It orchestrates all the other UI components. |
| **`index.css`** | Global styles for the application. |
| **`components/`** | This is where the individual UI elements live. Each component is a reusable piece of the interface. |
| ├─ `ChatArea.tsx` | The main chat window where you interact with the LLM. |
| ├─ `ChatMessage.tsx` | Represents a single message in the chat. |
| ├─ `ModelSelector.tsx` | The dropdown menu for selecting different AI models. |
| ├─ `Sidebar.tsx` | The navigation panel on the side of the application. |
| ├─ `Settings.tsx` | The settings panel. |
| ├─ `SetupWizard.tsx` | The step-by-step guide for first-time users. |
| ├─ `privacy/` | Components related to privacy features, like data export and consent management. See the "Compliance and Privacy" section for more details. |
| **`stores/`** | This folder is for "state management." State is the data that the application needs to keep track of (e.g., which model is selected, the chat history). |
| ├─ `appStore.ts` | The central "store" for the application's state, using a library called Zustand. |
| **`utils/`** | Contains helper functions that can be reused across the application. |
| ├─ `logger.ts` | A utility for logging information, which is helpful for debugging. |
| ├─ `updater.ts` | Logic for handling application updates. |
| **`types/`** | Contains custom data type definitions for TypeScript, which helps prevent bugs. |

---

## ⚙️ Backend (`src-tauri/`)

This is the Rust-powered backend that does all the heavy lifting.

| File/Folder | Description |
| :--- | :--- |
| **`src/`** | The source code for the Rust backend. |
| ├─ `main.rs` | The entry point of the backend application. It sets up the Tauri application and starts the backend services. |
| ├─ `commands.rs` | Defines the functions that the frontend can call. This is the bridge between the UI and the backend logic. |
| ├─ `llm_manager.rs` | Manages the lifecycle of the large language models (loading, unloading, etc.). |
| ├─ `rag_engine.rs` | Implements the Retrieval-Augmented Generation (RAG) logic, which allows the LLM to use external documents. |
| ├─ `pii_detector.rs` | The engine for detecting Personally Identifiable Information (PII) in text. |
| ├─ `presidio_bridge.rs` | A bridge to communicate with Microsoft Presidio, a data protection service. |
| ├─ `compliance/` | Contains logic related to regulatory compliance (e.g., GDPR, AI Act). See the "Compliance and Privacy" section for more details. |
| ├─ `security/` | Implements security features like chat encryption. |
| ├─ `database/` | Manages the application's internal database. |
| **`Cargo.toml`** | The Rust equivalent of `package.json`. It lists the backend's dependencies and project metadata. |
| **`tauri.conf.json`** | The main configuration file for the Tauri application. It defines the app's name, version, permissions, and how the frontend and backend are bundled together. |
| **`build.rs`** | A script that runs before the Rust code is compiled. It can be used for various build-time tasks. |
| **`migrations/`** | Contains SQL scripts for setting up and updating the database schema. |

---

## ⚖️ Compliance and Privacy

This application has been built with privacy and compliance at its core, adhering to the principles of the **GDPR (General Data Protection Regulation)** and the **EU AI Act**.

### Backend (`src-tauri/src/compliance/`)

The backend contains the core logic for enforcing compliance rules.

| File | Description |
| :--- | :--- |
| **`consent.rs`** | Manages user consent in a granular way, as required by **GDPR Article 7**. It handles granting, revoking, and tracking different types of consent (e.g., for PII detection, chat storage). |
| **`audit.rs`** | Implements a comprehensive audit trail system, logging all important events as required by **GDPR Article 30**. This is for accountability and for providing users with a history of how their data has been processed. |
| **`retention.rs`** | Manages data retention policies, ensuring that data is not stored for longer than necessary, in line with **GDPR Article 5**. It allows for configurable retention periods for different types of data. |
| **`commands.rs`** | Exposes the compliance features to the frontend. These are the functions that the UI calls to manage consent, view audit logs, export data, etc. |

### Frontend (`src/components/privacy/`)

The frontend provides the user interface for managing privacy settings.

| File | Description |
| :--- | :--- |
| **`PrivacyDashboard.tsx`** | The main privacy control center. It brings together all the other privacy components into a single, easy-to-use interface. |
| **`ConsentManager.tsx`** | The UI for managing user consent. It allows users to grant or revoke consent for different processing activities. |
| **`DataViewer.tsx`** | Allows users to view their personal data, in compliance with **GDPR Article 15 (Right of Access)**. |
| **`ExportPanel.tsx`** | Provides the functionality for users to export their personal data in various formats, as required by **GDPR Article 20 (Right to Data Portability)**. |
| **`DeletionRequest.tsx`** | Allows users to request the deletion of their personal data, in line with **GDPR Article 17 (Right to Erasure)**. |
| **`AuditTrail.tsx`** | Displays the audit trail, allowing users to see a history of how their data has been processed. |
| **`RetentionSettings.tsx`** | The UI for configuring data retention policies. |

### AI Act Compliance

The application also includes features to comply with the transparency and documentation requirements of the **EU AI Act**.

| File/Folder | Description |
| :--- | :--- |
| **`docs/AI_TRANSPARENCY_NOTICE.md`** | A detailed document that provides users with information about the AI system, its capabilities, limitations, and risks, as required by **AI Act Article 13**. |
| **`docs/model_cards/`** | This folder contains "model cards" for each of the supported AI models. These cards provide technical details about the models, their training data, and their performance, as required by **AI Act Article 53**. |
| **`src/components/TransparencyNotice.tsx`** | The UI component that displays the AI transparency notice to the user. This is shown on the first launch of the application and can be accessed at any time from the menu. |

---

## 🔧 Configuration Files

These files control how the project is built and run.

| File | Description |
| :--- | :--- |
| **`vite.config.ts`** | Configuration for Vite, the tool used to build and serve the React frontend during development. |
| **`tsconfig.json`** | Configuration for the TypeScript compiler. It tells the compiler how to check the code for errors. |
| **`tailwind.config.js`** | Configuration for Tailwind CSS, a utility-first CSS framework used for styling the application. |
| **`.gitignore`** | A list of files and folders that should be ignored by the Git version control system (e.g., temporary files, build outputs). |

---

## 📚 Documentation (`docs/`)

This folder is a treasure trove of information about the project.

| File/Folder | Description |
| :--- | :--- |
| **`ARCHITECTURE.md`** | An overview of the project's technical architecture. |
| **`MASTER_ROADMAP.md`** | The long-term plan for the project. |
| **`compliance/`** | In-depth documents about how the application complies with various regulations. |
| **`architecture/`** | Documents related to architectural decisions and evaluations. |
| **`model_cards/`** | Information about the different AI models supported by the application. |

We hope this guide helps you find your way around the BEAR-LLM project. If you have any questions, don't hesitate to ask!