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

The platform follows a hierarchical five-layer architecture:
1. Vision & Governance
2. Strategic Intelligence
3. Operational Management
4. Execution Engine
5. Autonomous Micro Agents

---

## Secure Local Database Foundation

Life OS implements a security-first local database foundation inside `src-tauri/src/db`:

1. **Dependency Provenance**: `rusqlite 0.31.0` -> `libsqlite3-sys 0.28.0` (`bundled-sqlcipher` feature), compiling SQLCipher 4.5.3 Community (built against SQLite 3.39.4).
2. **SQLCipher Encryption**: Page-level AES-256-CBC encryption with per-page HMAC-SHA512 integrity verification.
3. **Argon2id Key Derivation**: Master passcodes derive 256-bit database encryption keys using Argon2id ($m=65536, t=3, p=4$) and a 16-byte random salt.
4. **Atomic Salt Metadata Persistence**: Non-secret KDF metadata (`.db.kdf` JSON metadata file) is persisted alongside the database using atomic exclusive file creation (`O_CREAT | O_EXCL`), ensuring race-safe first-run initialization across concurrent processes/threads.
5. **Zero Double-KDF & Zeroizing Memory**: Derived keys are supplied to SQLCipher using raw-key syntax (`PRAGMA key = "x'<64_hex_digits>'";`), wrapped in `Zeroizing<String>` heap buffers to guarantee immediate memory sanitization when dropped.
6. **Deterministic Migrations**: Transactional version tracking via `_migrations` table with contiguous version validation (`1, 2, 3...`) on startup.
7. **Cryptographic Audit Log & Fail-Closed Startup**: Tagged length-prefixed SHA-256 hash chaining over all audit fields (`id`, `created_at`, `user_id`, `action_type`, `entity_name`, `entity_id`, `before_state`, `after_state`, `client_info`, `previous_hash`). Unambiguously distinguishes `None` from `Some("")`. Serialized atomically via SQLite `TransactionBehavior::Immediate` transactions. Audit log chain is verified at Tauri app startup (`init_app_database`), which fails closed if no passcode provider is available.

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
cargo test           # Run Rust unit tests
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
```

---

## Contributing

See `docs/CONTRIBUTING.md`.
