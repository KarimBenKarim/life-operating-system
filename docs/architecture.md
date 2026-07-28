architecture.md
# Life OS Software Architecture Specification

## 1. Executive Summary & Vision

**Life OS** is an open-source, private, and deeply integrated **Personal Operating System** designed to unify the scattered aspects of a user's life (tasks, notes, journals, habits, finances, health, and goals) into a single, cohesive **Digital Brain**.

Unlike existing point solutions that isolate user data into proprietary, disconnected silos, Life OS treats all aspects of life as a unified semantic network. It leverages a graph-based representation where every journal entry, habit track, expense record, and task is a node connected by rich, context-aware relationships. 

Importantly, Life OS is designed from the ground up for a **Human-AI Partnership**. It introduces an extensible, agent-native framework enabling specialized AI personas to securely collaborate with the user, detect correlations, and automate repetitive cognitive overhead (e.g., weekly reviews, budget analysis, sleep/mood insights).

To guarantee safety and permanence for the user's most intimate data over a 40-year horizon, Life OS adheres to a strict set of core design tenets:
- **Offline-First & Local-First**: The software runs fully locally. If cloud sync or cloud LLMs are used, they are optional, opt-in, encrypted, and abstracted behind strict local filters.
- **Data Sovereignty & Portability**: plain-text Markdown and local SQLite serve as primary data storage. There are no proprietary lock-ins.
- **Privacy-Centric Context Isolation**: Sensitive data is parsed and filtered locally. Highly sensitive data never leaves the local environment. Only summarized, low-entropy metadata is permitted to traverse cloud LLM gateways.

---

## 2. System Architecture & C4 Context

Life OS is structured according to the **C4 model** to provide clear abstractions from the high-level system boundaries down to component relations.

### System Context (Level 1)
At the highest level, the user interacts with the Life OS client applications (Desktop/Web/Mobile). The system communicates with local data repositories and securely interacts with optional external APIs (such as banks via Plaid, calendars via CalDAV, and LLMs via OpenAI/Anthropic/Ollama).

+------------------------------------------------------------+ | Life OS User | +------------------------------------------------------------+ | Interacts & Captures v +------------------------------------------------------------+ | Life OS System | +------------------------------------------------------------+ | | | Read/Write Retrieve/Log Optionally Sync v v v +---------------+ +---------------+ +-------------------+ | Local File | | Local Relational| | External Services | | System (MD) | | / Vector DB | | (Ollama, Plaid, | | | | (SQLite/HNSW) | | Webhooks, etc.) | +---------------+ +---------------+ +-------------------+


### Container Architecture (Level 2)
The internal structure consists of the following primary containers:
1. **User Interface (UI) Layer**: A React/TypeScript front-end packaged for Desktop (via Electron/Tauri) and Mobile (via Capacitor) or deployed as a self-hosted Web App.
2. **Core OS Engine**: A local Node.js/Rust backend that handles file monitoring, local search, database routing, graph index generation, and system orchestration.
3. **Storage Container**:
   - **Markdown Vault**: Local user directory storing journal files, structured templates, and notes.
   - **SQLite Database**: Used for rapid indexing, transactional integrity (finances, habits), and full-text search index cache.
   - **Vector Database**: Local embedded index (e.g., SQLite-vec or HNSWlib) for semantic search and retrieval-augmented generation (RAG).
4. **Agent & Intelligence Layer**: The AI core implementing the **Model Context Protocol (MCP)**, executing specialized local/remote agent personas, orchestrating prompt construction, memory retrieval, and local filtering.

---

## 3. Core Domain Models & Unified Graph Schema

Life OS avoids rigid tables in favor of a **Semantic Personal Knowledge Graph**. Every core domain is modeled as a specialized node with attributes (frontmatter or relational columns) and edges connecting them.

### Domain Entities

#### 1. Tasks & Projects (Task Domain)
- **Task**: An actionable item with states (`Inbox`, `Next`, `Scheduled`, `Completed`, `Archived`), priority, dates (due, start, completed), and optional recurrent rules.
- **Project**: A high-level outcome requiring multiple tasks. Has status, target date, and links to relevant journals or resources.

#### 2. Habits (Habit Domain)
- **Habit**: A repeating routine to build. Defines targets (frequency, metrics), cues, and rewards.
- **Habit Log**: A point-in-time record of a habit execution, capturing completion state, value (e.g., "5km run", "20 mins meditation"), and timestamp.

#### 3. Journals & Notes (Reflection Domain)
- **Journal**: A periodic entry (Daily, Weekly, Monthly, Quarterly, Annual) acting as a container for capture, sentiment, and retrospection.
- **Note**: A non-time-bound piece of information (ideas, recipes, bookmarks, books).

#### 4. Finances (Finance Domain)
- **Account**: Assets or liabilities (Bank, Cash, Credit, Investment).
- **Transaction**: Records movement of value. Attributes include amount, date, payee, category (income, expense, transfer), status (pending, cleared), and links to projects/habits (e.g., "Gym membership" linked to "Health Transformation" project).

#### 5. Health & Sleep (Biometrics Domain)
- **Metric Log**: Tracks biometrics (Sleep duration, sleep quality, heart-rate variability, mood rating, water intake, steps). Links closely with Daily Journals.

---

## 4. Technology Stack & Rationale

| Layer | Recommended Technology | Alternatives Considered | Rationale |
| :--- | :--- | :--- | :--- |
| **Front-End Framework** | **React (TypeScript) + Vite** | Vue, Svelte | Maximum ecosystem support for interactive graphs (d3/reactflow), rich text editors, and UI component libraries. TypeScript ensures type safety across domain interfaces. |
| **App Packaging** | **Tauri (Rust-based)** | Electron, Capacitor | Tauri produces lightweight, native binaries (< 10MB memory footprint vs. Electron's > 100MB) and embeds a high-performance Rust backend for file operations and database management, ensuring zero-latency offline performance. |
| **Relational Storage** | **SQLite (embedded)** | PostgreSQL, DuckDB | Zero setup, single-file database architecture aligned perfectly with offline-first design. Highly optimized for resource-constrained desktop and mobile environments. |
| **Vector Storage** | **SQLite-vec / HNSWlib (Local)** | Pinecone, Chroma | Fully local, fast vector search without external cloud dependencies, keeping semantic search entirely private. |
| **AI Agent Gateway** | **Model Context Protocol (MCP)** | Custom REST APIs | Standardizes how local tools, file access, and databases are exposed to LLMs. Provides a unified, secure, and vendor-agnostic interface for agent interactions. |
| **Local LLM Execution** | **Ollama** | Llama.cpp directly | Standardizes local inference, offering simple APIs and an active community for running models like LLaMA 3 or Mistral locally. |

---

## 5. Security & Privacy Posture

Because Life OS holds the totality of a user's digital footprint, security is paramount. The system adopts a **zero-trust local-first security architecture**:

### Data Isolation & Access Controls
1. **Local-Only Encryption**: Local database files (SQLite) and local cached indexes can optionally be encrypted at rest using AES-256 (via SQLCipher) using a user-derived master key.
2. **Context Masking**: The local Agent Gateway intercepts prompt payloads before they are sent to external APIs (OpenAI/Anthropic). It automatically scrubs:
   - High-entropy strings (passwords, API keys, routing numbers).
   - Highly sensitive categories designated by the user (medical terms, financial balances) and replaces them with abstract tokens or localized generalizations (e.g., replacing "$4,235.12" with "[High Balance Expense]").
3. **Local Embedding Models**: Sentence embeddings for semantic memory and RAG are computed fully locally on the client machine using ONNX-based lightweight transformer models. No raw note text is ever transmitted to remote APIs for indexing.

---

## 6. Implementation Roadmap

The implementation is broken down into four distinct, logical phases to ensure incremental value delivery and thorough testing.

+-----------------------------------------------------------------+ | Phase 1: Core Storage & Schema (Weeks 1-4) | | Setup SQLite, Markdown Vault Parser, and Relational Schema | +-----------------------------------------------------------------+ | v +-----------------------------------------------------------------+ | Phase 2: Core UI & Dashboard (Weeks 5-8) | | Build React Frontend, Graph Visualization, and Pomodoro Timer | +-----------------------------------------------------------------+ | v +-----------------------------------------------------------------+ | Phase 3: Agent Integration & MCP (Weeks 9-12) | | Implement MCP Server, Local LLM Integration, Memory, Pattern DB | +-----------------------------------------------------------------+ | v +-----------------------------------------------------------------+ | Phase 4: Production Sync & Tauri (Weeks 13-16) | | Tauri Packaging, Encrypted Sync (CRDTs), and Mobile Build | +-----------------------------------------------------------------+


### Phase 1: Core Storage & Schema (Weeks 1-4)
- Set up Tauri codebase with Rust backend and embedded SQLite.
- Create Markdown Vault Parser, translating Frontmatter YAML and raw markdown notes into transactional relational indexes on file system changes.
- Establish strict schema validations and database migrations for Tasks, Habits, Finances, and Journals.
- **Milestone 1**: A functioning CLI and background daemon that can parse a directory of Markdown files and generate a relational SQLite graph index successfully.

### Phase 2: Core UI & Dashboard (Weeks 5-8)
- Build the React-based frontend and dashboard UI.
- Implement the interactive "Brain Graph" using React Flow / D3.js, demonstrating live linking of notes, tasks, and transactions.
- Integrate the Focus (Pomodoro) Timer, Habit heatmap tracker, and Financial Ledger.
- Implement "Quick Capture" (CMD+K) modal allowing rapid classification of incoming data.
- **Milestone 2**: A fully offline, visual desktop application capable of managing tasks, tracking habits, plotting charts, and visualizing notes in a network diagram.

### Phase 3: Agent Integration & MCP (Weeks 9-12)
- Develop the local MCP (Model Context Protocol) Server exposing file system actions, SQLite read-only search, and graph querying tools.
- Set up local Ollama routing and configure prompt templates for the Cabinet (specialized agent personas like Finance Coach, Sleep Analyst, Weekly Retrospective Agent).
- Implement Context Masking logic to prevent raw, high-sensitivity data leakage.
- Build Local RAG engine with native embedding extraction.
- **Milestone 3**: AI Agent panel inside the UI where the user can chat with the local model, ask for sleep correlation patterns, and generate a draft Weekly Review note automatically based on the last 7 days of logs.

### Phase 4: Production Packaging & End-to-End Verification (Weeks 13-16)
- Package desktop application using Tauri for macOS, Windows, and Linux.
- Set up optional encrypted end-to-end sync using Yjs/CRDTs over WebRTC or private self-hosted bucket.
- Perform thorough end-to-end performance and memory audits, ensuring memory footprint stays below 50MB and local graph querying resolves within < 10ms.
- **Milestone 4**: Stable v1.0.0 released with complete automated installers, native packages, and user documentation
