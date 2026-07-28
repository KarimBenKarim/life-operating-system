# ADR 0003: Data Storage and Privacy Architecture

## Status
Accepted

## Context
Life OS processes highly intimate, sensitive, and private data, including financial transactions, daily mood logs, personal journals, and daily biometrics. To guarantee privacy and ensure a lifetime (40+ years) of durability, we must design an offline-first storage model that protects users from corporate cloud service bankruptcies, API deprecations, or security breaches.

## Decision
We select a hybrid storage model: **Plain-text Markdown files** (as the permanent document vault) + **Local SQLite Database** (as the high-performance relational cache and indexing store).

### Markdown Note Vault
- Notes, ideas, journals, and goals are saved directly to local `.md` files.
- Metadata is stored in a clean frontmatter block (YAML format).
- The markdown vault is the *source of truth*. If the SQLite database is deleted or corrupted, it can be fully reconstructed by re-scanning the vault.

### SQLite Cache
- Tasks, habit streaks, financial transactions, and index relationships are mirrored in an embedded local SQLite database.
- Read operations and dashboard metric computations (charts, statistics) pull directly from SQLite for sub-millisecond response times.
- Background Rust file-watchers sync markdown updates to SQLite in real-time.

### Privacy-Preserving Agent Gateway
- To prevent leaking sensitive information to external LLM services, we introduce local **Context Masking**.
- Prompts are processed locally: high-entropy data (e.g., precise account numbers, addresses, balances) are replaced with abstract tokens (e.g., `[SCRUBBED_ACCOUNT_1]`) before being sent to public LLM endpoints.
- Semantic embedding generation (vector indexing) runs exclusively on local hardware using ONNX-based transformer models.

## Rationale
- **Durability**: Markdown is future-proof. Even if the Life OS software becomes obsolete, the user's data can still be read by any standard text editor decades from now.
- **Offline Sovereignty**: Zero cloud storage is required. Users can optionally choose to sync their Markdown folder via standard, end-to-end encrypted protocols (like Syncthing, Git, or iCloud).
- **Security**: The zero-trust posture guarantees that personal secrets never leak into public LLM training datasets.

## Consequences
- **Positive**: Total privacy, physical ownership of data, zero cloud bills, and instantaneous local query performance.
- **Negative**: The client app must handle complex file conflict resolutions if the user edits their Markdown files in external editors.
