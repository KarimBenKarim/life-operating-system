# ADR 0022: Local-First Encryption at Rest via SQLCipher and AES-256

## Status
Accepted

## Context
Life OS stores intimate, highly sensitive user telemetry (including biometric sleep tracks, resting heart rates, detailed double-entry financial transactions, personal goals, and historical notes). Since Life OS operates primarily as a local desktop-first application (utilizing Tauri and an embedded database), leaving these records in raw plaintext on disk creates a major security vulnerability where other local users or malicious scripts on the host device could easily inspect or hijack the user's data.

## Decision
We select **SQLCipher (AES-256-GCM)** as our mandatory local database encryption library for all Relational (SQLite) and Vector databases:
1. **Passcode Fallback**: The database key is derived from the user's master passcode.
2. **Key Derivation Pipeline**: We pass the passcode through **Argon2id** (configured with standard recommended params: $m=65536$, $t=3$, $p=4$) alongside a local cryptographic salt to compute a 256-bit derived key.
3. **Decryption**: The key is passed directly to SQLCipher on app startup to unlock the database in memory. No decrypted databases or raw passcodes are ever written to disk.

## Rationale
- **Zero-Knowledge Privacy**: Since the master passcode is never stored on disk and the derived key is held only in transient memory, the database files are completely unreadable to unauthorized local actors, ensuring data sovereignty.
- **Performance**: SQLCipher executes AES encryption directly at the page level during database page reads/writes, causing close-to-zero noticeable user interface delay.
- **Industry Standard**: AES-256-GCM is globally recognized as highly secure, ensuring absolute resistance against brute-force attacks.

## Consequences
- **Positive**: Complete privacy of personal/financial data, zero local database readability without biometrics/passcode, and zero metadata leakage.
- **Negative**: If the user permanently loses their master passcode and does not possess a backup passcode, the database is completely irrecoverable, adhering strictly to the privacy-first model.
