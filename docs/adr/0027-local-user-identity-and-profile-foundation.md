# ADR 0027: Local User Identity & Profile Foundation Architecture

## Status
Accepted

## Context
Life OS is a private, offline-first personal operating system designed for human-AI partnership. In accordance with ADR 0003 and ADR 0022, all domain data and system activity must be anchored to a sovereign local identity without reliance on external cloud authentication services or centralized identity providers.

Following the local database engine (Task 13) and local Markdown vault persistence boundary (Task 14), we must establish the canonical local user identity and profile persistence domain that future API layers, event channels, and agent frameworks can depend upon.

This ADR answers: "Who is the local Life OS user, and where is that identity/profile persisted?"
It intentionally defers answering: "How does the user authenticate?"

## Decision

### 1. Canonical Local User Profile Entity & Schema
Life OS establishes a canonical local user profile record stored in the encrypted SQLCipher database under Migration V3 (`V3__user_identity_and_profile`):

```sql
CREATE TABLE IF NOT EXISTS user_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT,
    display_name TEXT NOT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_profiles_created_at ON user_profiles(created_at);
```

### 2. Immutable User ID Invariant
- **UUID v4 Identifier**: Every local user profile is assigned a unique UUID v4 stable ID upon creation.
- **Primary Key Immutability**: The primary key `id` is strictly immutable. Profile update operations modify profile attributes (`display_name`, `email`, `timezone`, `updated_at`) without altering or re-assigning the primary key `id`.
- **Single-User Local Sovereignty Model**: In accordance with the local desktop application architecture, Life OS enforces a single active local user profile per encrypted database workspace. Attempting to create duplicate local profiles returns an `AlreadyExists` domain error.

### 3. Encrypted Database Protection
- User profile data resides exclusively inside the encrypted SQLCipher 4.5.3 database file (`lifeos.db`), protected by AES-256-CBC encryption at rest with 256-bit keys derived via Argon2id ($m=65536, t=3, p=4$).
- User profile metadata is never written in plaintext to external files or unencrypted caches.

### 4. Single-Transaction Cryptographic Audit Integration
- Mutations to the user profile trigger cryptographic system audit log entries (`USER_PROFILE_CREATE`, `USER_PROFILE_UPDATE`).
- **Atomic SQLite Transaction**: Metadata table updates (`user_profiles`) and corresponding audit log entries (`system_audit_logs`) execute and commit within **ONE single immediate SQLite transaction** (`log_audit_event_tx`). If audit log insertion fails, the profile mutation rolls back atomically.
- **Privacy & Credential Safeguard**: Audit log entries record profile metadata state changes (display name, email, timezone, timestamps). Audit records **never** contain passwords, passcodes, derived keys, keyring secrets, or authentication tokens.

### 5. Explicit Deferrals (Future Scope)
To preserve strict domain boundaries, the following authentication and session components are explicitly deferred to subsequent tasks:
- Host OS Keyring integration (`keyring` crate / Keychain / Credential Manager).
- Automatic database passcode retrieval and unlock UI flows.
- Application lock/unlock session tokens.
- JWT access tokens, refresh tokens, and HttpOnly cookie rotation.
- Role-based access control (RBAC) and third-party OAuth providers.

## Consequences
- **Positive**: Establishes a clean, auditable, immutable local user identity anchor; enforces single-transaction SQLite consistency; guarantees zero credential leakage; and respects offline local sovereignty.
- **Negative**: The application bootstrap currently retains the temporary environment variable passcode fallback (`LIFEOS_DB_PASSCODE`) until the OS Keyring unlock task is formally implemented.
