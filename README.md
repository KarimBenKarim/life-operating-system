# Life Operating System (Life OS)

## Overview

Life OS is an AI-native platform designed to function as an Executive Operating System for managing every aspect of a person's life over multiple decades.

Rather than acting as a traditional productivity application, Life OS serves as an autonomous executive organisation composed of multiple AI agents working together across strategic, operational and execution layers.

The objective is to maximise long-term success while maintaining balance between career, health, research, finance, relationships, learning, spirituality and personal wellbeing.

---

## Vision & Architecture

- Vision: `docs/vision.md`
- Architecture: `docs/architecture.md`
- Roadmap: `docs/roadmap.md`
- Architecture Decision Records (ADRs): `docs/adr/`
- Database Architecture: `docs/database-architecture.md`
- Security Architecture: `docs/security-architecture.md`

The platform follows a hierarchical five-layer architecture:
1. Vision & Governance
2. Strategic Intelligence
3. Operational Management
4. Execution Engine
5. Autonomous Micro Agents

---

## Database & Security Architecture (Task 13 Implementation)

### Database Location & Resolution
The local SQLite/SQLCipher database (`lifeos.db`) is resolved dynamically at runtime using the host OS app data directory (via Tauri `path().app_data_dir()`). Database files are strictly excluded from git tracking via `.gitignore`.

### Encryption at Rest (SQLCipher & Argon2id)
- **Engine**: SQLCipher with page-level AES-256-GCM encryption.
- **Key Derivation (Argon2id)**: User master passcode is derived using Argon2id ($m=65536$ 64MB memory, $t=3$ iterations, $p=4$ parallelism, 32-byte key output) with a 16-byte random salt.
- **Zeroization**: Derived key material is wrapped in zeroizing structures (`ZeroizeOnDrop`) to ensure keys are wiped from transient memory when dropped.
- **No Hardcoded Keys / Secrets**: Raw passcodes and decrypted keys are never written to disk or logged.

### Schema Migration Engine
Versioned schema migrations are executed deterministically in transaction blocks. The `schema_migrations` tracking table records applied migration versions and timestamps, ensuring idempotent application.

### Cryptographic Audit Log
System audit logs (`system_audit_logs`) feature an immutable append-only hash chain. Each row signature is computed as:
$$Hash_n = SHA256(id_n + timestamp_n + action_n + prev\_hash_{n-1} + after\_state\_hash_n)$$
An automated verification engine checks the entire hash chain from genesis (`0000...0000`) on application startup, returning `DbError::AuditTampering` if past logs, signatures, timestamps, or deleted entries are detected.

---

## Developer Guide & Executable Setup

### Prerequisites

- **Node.js**: `^22.0.0`
- **npm**: `^10.0.0` or `^11.0.0`
- **Rust Toolchain**: `^1.80.0` or higher (`cargo`, `rustc`)
- **System Dependencies (Linux/Ubuntu)**:
  `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`, `pkg-config`

---

### Quick Start

#### 1. Install Frontend Dependencies
```bash
cd frontend
npm install
```

#### 2. Run Frontend in Development Mode (Vite)
```bash
cd frontend
npm run dev
```

#### 3. Run Tauri Desktop App in Development Mode
```bash
npm run tauri dev # Or from root using npx/tauri CLI
```

---

### Verification & Testing Commands

#### Frontend Quality Gates
```bash
cd frontend
npm run typecheck    # TypeScript compilation check
npm run lint         # ESLint checks
npm test             # Vitest unit & smoke tests
npm run build        # Production bundle build
```

#### Backend Quality Gates
```bash
cd src-tauri
cargo fmt --check    # Check Rust formatting
cargo clippy         # Run Clippy static analysis
cargo test           # Run Rust unit & database/security tests
cargo check          # Build check Rust crate
```

#### Running CI Quality Gates Locally
To run all CI checks locally prior to committing:
```bash
(cd frontend && npm run typecheck && npm run lint && npm test && npm run build) && \
(cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo check)
```

---

## Repository Structure

```text
.
├── .github/workflows/ci.yml   # GitHub Actions CI quality gates
├── AGENTS.md                  # Master Engineering Handbook
├── README.md                  # Developer & Architecture Guide
├── docs/                      # Architectural specifications & 33 ADRs
├── frontend/                  # React + TypeScript + Vite UI frontend
└── src-tauri/                 # Rust + Tauri desktop core backend
    └── src/
        ├── db/                # SQLCipher, Argon2id, Migrations & Audit Log
        └── lib.rs             # Tauri entry point & module registrations
```

---

## Contributing

See `docs/CONTRIBUTING.md`.
