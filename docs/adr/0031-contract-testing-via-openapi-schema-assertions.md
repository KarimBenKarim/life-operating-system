# ADR 0031: Contract Testing via OpenAPI Schema Assertions

## Status
Accepted

## Context
As the Life OS codebase grows, multiple client applications (Tauri, Capacitor, external self-hosted browser widgets) will consume the core API Gateway. If an API route is updated, a database schema changes, or a response key is modified without aligning client SDKs, it can trigger silent, critical runtime crashes. Standard integration tests usually only assert HTTP status codes (like `200` or `201`), failing to detect subtle schema key changes.

## Decision
We select **Contract-First Testing via OpenAPI Schema Assertions**:
1. **Contract-First Validation**: All integration tests (using libraries like `jest-openapi` or `chai-openapi`) must dynamically validate the keys and value types of REST and WebSocket payload responses directly against our central OpenAPI 3.1 contract (`docs/api-architecture.md`).
2. **Strict Matching**: Any API payload returned by a controller that deviates from the contract (such as a missing field, an extra un-specified parameter, or a mismatched type like returning an integer instead of a string UUID) will forcefully fail the integration test suite.

## Rationale
- **Prevention of API Drift**: Validating payloads against the OpenAPI specification guarantees that our code contract and our active codebase remain 100% synchronized, eliminating silent API drift.
- **Type-Safety Enforcement**: Forcing server responses to strictly match the schema allows automated frontend generators (like `openapi-typescript`) to compile absolute, compile-time type-safe API clients, eliminating client-side decoding bugs.

## Consequences
- **Positive**: Eradicates API/frontend drift bugs, enforces compile-time type-safety, and provides immediate feedback if database model migrations alter API payloads.
- **Negative**: Requires developers to keep the OpenAPI contract spec updated first before any backend route schema modifications are checked in.
