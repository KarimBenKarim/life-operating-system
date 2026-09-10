# ADR 0026: Local Markdown Vault & Persistence Boundary Architecture

## Status
Accepted

## Context
Life OS prioritizes human data sovereignty and long-term durability (40+ years). In accordance with ADR 0003, personal knowledge, journals, reflections, and notes are stored as local plain-text UTF-8 Markdown files with YAML frontmatter.

To support high-performance UI rendering, graph queries, and cryptographic auditability, Life OS maintains an encrypted SQLite metadata index (`vault_documents`) alongside the cryptographic system audit log (`system_audit_logs`).

We must formally establish the architecture, boundaries, security posture, atomic persistence mechanics, and consistency models between the filesystem Markdown vault and the encrypted SQLite storage subsystem.

## Decision

### 1. Clear Storage Subsystem Boundary
Life OS strictly distinguishes between two distinct persistence layers:
- **Plaintext Local Markdown Vault**: UTF-8 Markdown files stored in the local filesystem under the application data vault root. Each document includes YAML frontmatter with stable ID, schema version, and metadata. These files are **not encrypted** by SQLCipher; they remain human-readable plain-text to ensure permanent durability and external tool compatibility.
- **Encrypted SQLCipher SQLite Database**: Encrypted database containing the relational cache, the document metadata index (`vault_documents`), and the immutable system audit log chain (`system_audit_logs`).

### 2. Path Safety & Symlink Escape Prevention
Document access and mutation through the vault abstraction strictly enforce path isolation (`validate_and_resolve_relative_path`):
- **Relative Path Requirement**: Absolute paths (POSIX `/...`), Windows drive letters (`C:\...`), UNC network paths (`\\server\share`), and NT device namespaces (`\\.\`, `\\?\`) are strictly rejected.
- **Traversal Prevention**: Path components containing `..`, `.`, or null bytes (`\0`) are rejected. Backslash separators (`\`) are normalized to prevent cross-platform bypasses.
- **Symlink Escape Mitigation**: Target paths and existing ancestor directories are canonicalized and verified to reside within the canonicalized vault root directory. Attempting to traverse or escape through symlinks or reparse points returns a `SymlinkEscape` error.

### 3. Atomic Filesystem Persistence
File modifications (writes and updates) execute using atomic write semantics (`write_atomic`):
1. A temporary file is created in the same target directory using a unique name (`.tmp_<uuid>.tmp`).
2. Data is written to the temporary file, followed by explicit buffer flush (`file.flush()`) and disk sync (`file.sync_all()`).
3. File handles are closed before performing an atomic rename (`fs::rename`) replacing the target file.
4. On any failure prior to completion, temporary files are removed, preserving the original target file intact.

### 4. Cross-Subsystem Consistency & Reconciliation Model
Because operating system filesystems and SQLite databases operate on separate storage engines without native cross-system distributed transactions:
- **No Global Multi-Subsystem Transactional Atomicity**: Life OS does not claim native ACID atomicity spanning both the filesystem and SQLite simultaneously.
- **Crash Consistency Sequence**: During document mutation, the physical Markdown file is written atomically first, followed immediately by the SQLite metadata update and audit log event append.
- **Vault Re-Indexing Strategy**: The filesystem Markdown vault remains the single source of truth. If a crash or power failure occurs between file write and database commit, the encrypted SQLite index can be fully reconciled and rebuilt by re-scanning the Markdown files in the local vault.

### 5. Cryptographic Audit Logging
- Document creations, updates, and deletions trigger cryptographic audit log entries (`VAULT_DOCUMENT_CREATE`, `VAULT_DOCUMENT_UPDATE`, `VAULT_DOCUMENT_DELETE`).
- Audit log entries record document metadata, relative paths, file sizes, and SHA-256 content hashes.
- **Privacy Assurance**: Raw Markdown body text is **never** written to the system audit log.

## Platform Limitations & Security Considerations
- **Time-of-Check to Time-of-Use (TOCTOU)**: On standard OS filesystems, non-atomic directory modifications outside application control could theoretically alter symlinks between path validation and file open. Users are advised not to grant untrusted background processes write access to the vault root directory.
- **Atomic Rename Semantics**: On POSIX filesystems, `rename` within the same filesystem directory is guaranteed atomic. Writing temporary files in the target directory avoids cross-filesystem move non-atomicity. On Windows, target replacement is supported via OS file system APIs.

## Consequences
- **Positive**: Total user data sovereignty, zero proprietary lock-in, atomic file writes preventing partial corruption, and auditable metadata tracking.
- **Negative**: Lack of native two-phase commit between OS filesystem and SQLite requires automated reconciliation routines if the application crashes during document saving.
