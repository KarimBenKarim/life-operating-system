AGENTS.md
# AGENTS.md

Welcome, AI Agent! This file contains instructions, tips, and guidelines for working with the **Life OS** codebase. As a personal operating system designed for human-AI partnership, Life OS prioritizes privacy, offline-first data sovereignty, extensibility, and semantic graph-based connectivity.

## Scope of AGENTS.md
The scope of this file is the entire Life OS repository. Any agent modifying or exploring any file in this repository must adhere to the conventions and guidelines specified here.

---

## Core Philosophy

1. **Human-AI Partnership**: Life OS is not a passive tool or a black-box automation engine. It is an infrastructure designed for active, long-term collaboration between a human user and their personal AI agents.
2. **Offline-First & Local Sovereignty**: All user data (notes, journals, tasks, finances, habits, etc.) must remain primarily under the user's control. Plain-text markdown files, local SQLite databases, or local vector search indices are preferred over proprietary cloud storage.
3. **Everything Connects (Graph-Based)**: Life OS treats life as a unified semantic network rather than isolated silos. Tasks, habits, journals, and finances are nodes in a single, deeply connected personal knowledge graph.
4. **Agent-Native Architecture**: Built from the ground up to allow specialized AI agents (e.g., Finance Agent, Health & Sleep Agent, Habit Agent) to securely access contextual information and help automate daily reflection, planning, and pattern detection.

---

## Coding Conventions & Guidelines

When implementing application code or modifying the repository in subsequent phases, adhere strictly to these principles:

### 1. Data Formats
- **Plain-Text Markdown**: Use Frontmatter (`yaml`) for structured metadata.
- **Strict Schema Enforcement**: Any automated sync, database migrations, or JSON exports must adhere to versioned schemas.
- **Relational Integrity**: If local SQLite or embedded relational storage is used, enforce strict foreign key constraints.

### 2. Privacy & Security Constraints
- **Context vs. Sensitivity**: Never leak or send highly sensitive data (like unmasked bank credentials, medical diagnostic details, or raw private keys) to public AI APIs.
- **Local Summarization**: Perform pre-processing, filtering, and summarization of sensitive records locally before passing summaries or abstract metadata to cloud-hosted LLM endpoints.
- **Local Models First**: Prioritize the use of local LLMs (e.g., via Ollama/Llama.cpp) and local embedding models (e.g., ONNX, HuggingFace transformers) where feasible.

### 3. Modularity & Agent Interfaces
- **Specialized Interfaces**: AI agents must interact with the Core OS through strict, well-defined CLI, API, or MCP (Model Context Protocol) boundaries.
- **Dry-Run Default**: Any agent action that modifies state (e.g., modifying a user's calendar, moving funds, bulk editing files) must support a dry-run mode or require explicit user confirmation.

---

## Verification & Testing Requirements

1. **No Artifact Editing**: Never edit generated artifacts under `dist/`, `build/`, or static bundles directly. Always modify source code and execute the build tools.
2. **Validate Markdown & Syntax**: Ensure that all markdown documentation, ADRs, and diagrams use valid syntax (e.g., correct Mermaid diagram structures, valid frontmatter).
3. **Write Tests**: For any application code added in future phases, write unit tests and integration tests demonstrating correctness, edge-case coverage, and offline compatibility.
