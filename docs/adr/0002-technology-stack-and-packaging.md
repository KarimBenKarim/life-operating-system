
# ADR 0002: Technology Stack and Packaging

## Status
Accepted

## Context
Life OS requires a client runtime that runs locally on desktop platforms (macOS, Windows, Linux) and can optionally be extended to mobile. It must support high-performance operations (such as file-watching, SQLite transactional queries, and local vector indexing) with a minimal memory footprint and close-to-zero latency.

## Decision
We select **Tauri** (Rust backend wrapper) combined with a **React (TypeScript)** + **Vite** frontend web view.

### Frontend
- **React**: Highly mature, rich component ecosystem (particularly for data visualization and graphing).
- **TypeScript**: Enforces strict domain typing.
- **Vite**: Ultra-fast bundler and hot module replacement.

### Native Packaging & Backend
- **Tauri**: Replaces heavy Chromium-based Electron shells with light, native OS web viewers. Uses Rust for the backend container.
- Tauri IPC handles intensive local tasks (SQLite read/write, file watching, background scheduling) natively in Rust.

## Rationale
- **Resource Constraints**: Electron apps regularly consume 150MB+ idle RAM. Tauri apps compile down to < 10MB binaries and run on 15-30MB of idle RAM, ensuring Life OS can run continuously in the background without affecting system performance.
- **Security**: Tauri restricts frontend API access by default, preventing malicious third-party plugins or dependencies from freely reading/writing arbitrary parts of the user's hard drive.
- **Performance**: High-intensity computations, local relational index maintenance, and markdown parsing are implemented in Rust (via Tauri Commands) ensuring instantaneous interface updates.

## Consequences
- **Positive**: Blazing fast startup, tiny executable size, maximum security sandboxing, and access to Rust's rich library ecosystem for file tracking and memory safety.
- **Negative**: Requires developers to understand both TypeScript and Rust when writing complex system-level integrations.
