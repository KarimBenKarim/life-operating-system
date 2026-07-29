# ADR 0030: Unified Testing Pyramid and Framework Selection

## Status
Accepted

## Context
Life OS contains distinct execution contexts: a React frontend, a Tauri Rust core, an embedded SQLCipher database, and a local/cloud AI agent gateway. To maintain absolute system reliability over a multi-decade horizon, we must select an automated, cohesive, and comprehensive testing framework. Relying solely on manually written test scripts or un-structured testing scopes leads to software regressions, slow release gates, and brittle components.

## Decision
We select a unified **Testing Pyramid** and explicit, industry-standard frameworks to isolate and validate logical layers:
- **Unit Testing**: **Vitest** (for React, custom hooks, and Zustand store slices) + Rust's native **`cargo test`** framework (for Tauri Rust core commands, file parsers, and database helpers).
- **Integration Testing**: **Supertest** + **MSW (Mock Service Worker)** for mocking remote APIs, and Tauri's native `mock_handler` for IPC boundary verification.
- **End-to-End UI Testing**: **Playwright** with the native Playwright-Tauri driver to automate full-screen visual desktop client actions.

## Rationale
- **Vitest and RTL**: Vitest provides ultra-fast test runs and has direct compatibility with Vite's bundler, allowing tests to run with sub-second reload times during development.
- **Playwright-Tauri Integration**: Playwright is the gold standard for E2E web automation. The Tauri Playwright driver allows us to run test journeys directly inside the compiled desktop app shell, simulating real user interaction constraints (like drag-and-drop kanban boards or canvas animations).
- **Mock Service Worker (MSW)**: MSW intercepts network requests at the browser-network layer, enabling components to test real axios/fetch networks without spinning up live cloud-hosted mock servers.

## Consequences
- **Positive**: Exceptional coverage boundaries, rapid local test runs, signed visual E2E verification, and a completely standardized testing stack.
- **Negative**: Requires developers to master both Vitest (JS/TS) and Cargo Test (Rust) paradigms.
