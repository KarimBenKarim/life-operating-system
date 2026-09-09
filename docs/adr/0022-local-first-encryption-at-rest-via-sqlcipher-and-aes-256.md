# ADR 0022: Local-First Encryption at Rest via SQLCipher and AES-256

## Status
Accepted (Updated)

## Context
Life OS stores intimate, highly sensitive user telemetry (including biometric sleep tracks, resting heart rates, detailed double-entry financial transactions, personal goals, and historical notes). Since Life OS operates primarily as a local desktop-first application (utilizing Tauri and an embedded database), leaving these records in raw plaintext on disk creates a major security vulnerability where other local users or malicious scripts on the host device could easily inspect or hijack the user's data.

## Decision
We select **SQLCipher 4.5.3 Community** (built against SQLite 3.39.4) as our mandatory local database encryption library for all Relational (SQLite) and Vector databases:
1. **Passcode Source**: The primary entropy source for database key derivation is the user's master passcode. Passcodes are never persisted to disk or logged.
2. **Key Derivation Pipeline (Argon2id)**:
   - Application-level key derivation executes via **Argon2id** configured with parameters: $m=65536$ KiB (64 MiB), $t=3$ iterations, $p=4$ parallelism threads, and a 16-byte (128-bit) cryptographically random salt.
   - Non-secret KDF parameters and salt metadata are persisted alongside the database in a JSON file (`<db_path>.kdf`) to allow deterministic key recovery across application restarts.
3. **Raw Key Handoff**:
   - The derived 256-bit binary key is handed directly to SQLCipher on application startup using raw-key syntax (`PRAGMA key = "x'<64_hex_digits>'";`).
   - This raw-key handoff explicitly instructs SQLCipher to use the 256-bit key directly as its master key, bypassing SQLCipher's internal secondary PBKDF2 string passphrase derivation.
4. **Cipher Architecture**:
   - SQLCipher 4.x executes page-level encryption using **AES-256-CBC** combined with per-page **HMAC-SHA512** integrity verification and random per-page IVs.
5. **Memory Zeroization**:
   - Derived key material in memory is held within `Zeroizing<T>` containers to ensure immediate memory sanitization when dropped.

*Specification Correction Note*: Initial drafts referenced AES-256-GCM. SQLCipher 4.x natively implements page-level AES-256-CBC with HMAC-SHA512. This ADR update accurately records the exact SQLCipher 4.x architecture compiled in the platform, which provides robust page-level confidentiality and authentication without compromising the security model.

## Rationale
- **Zero-Knowledge Privacy**: Since the master passcode is never stored on disk and the derived key is held only in transient zeroizing memory, database files are completely unreadable to unauthorized local actors, ensuring data sovereignty.
- **Performance & Authenticated Encryption**: SQLCipher executes AES-256-CBC encryption directly at the page level with HMAC-SHA512 authentication during page reads/writes, causing close-to-zero noticeable user interface delay while preventing page-tampering attacks.
- **Industry Standard**: SQLCipher 4.x is globally recognized as highly secure, ensuring absolute resistance against brute-force attacks and physical disk inspection.

## Consequences
- **Positive**: Complete privacy of personal/financial data, zero local database readability without biometrics/passcode, zero metadata leakage in WAL files (`.db-wal` frame pages are encrypted using the same page key/HMAC).
- **Negative**: If the user permanently loses their master passcode and does not possess a backup passcode, the database is completely irrecoverable, adhering strictly to the privacy-first model.
