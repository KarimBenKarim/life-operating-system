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
