# ADR 0026: Local Markdown Vault and Atomic Persistence Boundary

## Status
Accepted

## Context
Life OS processes intimate, private user data over a multi-decade horizon. Per ADR 0003, plain-text Markdown files serve as the primary source of truth, while an encrypted SQLite database acts as the high-performance relational cache and metadata index. To prevent data corruption, directory escapes, and un-audited file mutations, the platform requires a safe, deterministic, and isolated vault persistence boundary.

## Decision
We establish a local **Markdown Vault and Atomic Persistence Boundary** in Rust (`src-tauri/src/vault`):
1. **Configurable Vault Root & Path Safety**:
   - Vault paths are strictly partitioned into `VaultRoot`, `RelativePath`, and `ResolvedPath`.
   - `PathValidator` enforces boundary limits: rejecting absolute path injections (`/etc/passwd`, `C:\...`), parent directory traversals (`..`), platform-specific trick separators (`\`), and symlink/reparse-point escape attempts.
2. **Markdown Document & YAML Frontmatter Model**:
   - Every document maintains a YAML frontmatter block containing `id` (UUIDv4), `title`, `created_at`, `updated_at`, `schema_version`, and flattened `extra` map (preserving unknown metadata fields).
   - UTF-8 text bodies and multiline YAML strings are supported. Missing frontmatter defaults gracefully without data loss.
3. **Atomic Write Strategy**:
   - Writes write to a temporary file (`.tmp_<uuid>`) inside the target file's parent directory, flush and sync to disk (`file.sync_all()`), and atomically rename (`fs::rename`) to the target path.
   - On write/rename errors, temporary files are removed immediately, ensuring the existing file on disk remains unharmed.
4. **SQLite Metadata Indexing**:
   - Migration V2 creates the `vault_documents` metadata tracking table in SQLite.
   - Document metadata (`id`, `relative_path`, `title`, `file_size_bytes`, `content_hash`, timestamps) is synced to SQLite on creation/update and removed on deletion.
5. **Cryptographic Audit Trail**:
   - Vault operations trigger Task 13 cryptographic audit events (`vault.document_create`, `vault.document_update`, `vault.document_delete`).
   - Raw Markdown body text is excluded from audit logs to protect privacy; only metadata and SHA-256 content hashes are logged.

## Rationale
- **Data Sovereignty & Durability**: Plain-text Markdown remains future-proof across decades, independent of proprietary database drivers or cloud vendor availability.
- **Data Integrity & Protection**: Atomic temporary file replacement prevents partial write file corruption. Path validation prevents local path traversal vulnerabilities.
- **Privacy & Auditability**: Chained audit trails provide complete operational accountability without exposing sensitive document content inside system logs.

## Consequences
- **Positive**: Robust path safety, zero data corruption on write failures, full offline privacy, and complete audit trail integrity.
- **Negative**: Adds a minor filesystem I/O overhead during atomic temp-file creation and rename operations.
