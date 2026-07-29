# ADR 0027: GitHub Actions for Automated CI/CD and Tauri Packaging

## Status
Accepted

## Context
Life OS targets multiple client operating systems (macOS, Windows, Linux desktop wrappers) and server-side Docker hosts. Packaging cross-platform native installers manually on local developer machines is prone to compiler mismatch errors, lacks security audibility, and slows down release cycles. We need a secure, cloud-hosted automation runner to build, test, audit, and package all client and server assets.

## Decision
We select **GitHub Actions** as the mandatory CI/CD automation pipeline platform:
1. **Quality Check Gates**: Every pull request triggers concurrent ESLint formatting checks, Rustfmt audits, and unit/contract test suites.
2. **SAST Security Scanning**: The workflow incorporates `cargo-audit`, `npm audit`, and `trivy` container scanning to verify that dependencies are free from known security vulnerabilities.
3. **Cross-Platform Tauri Builds**: On release tags, GitHub Actions triggers concurrent native virtualization runners (macOS, Windows, Ubuntu) to compile signed native Tauri installers (`.dmg`, `.msi`, `.deb`).
4. **Server Image Push**: Compiles the backend production images and pushes them to the secure, private GitHub Container Registry.

## Rationale
- **Declarative Workflows**: GitHub Actions uses simple, version-controlled YAML files located in `.github/workflows/`, making pipeline changes transparent and trackable.
- **Concurrent Native Runners**: Offloading Tauri compilation to GitHub's native environment runners ensures clean, repeatable builds, completely bypassing local environment configuration differences.
- **Deep Integration**: Integrates directly with GitHub Repository secrets (for signing certs) and release tags, drafting release payloads automatically.

## Consequences
- **Positive**: Cryptographically signed cross-platform native installers, automated dependency vulnerability alerts, and rapid release-to-production cycles.
- **Negative**: Adds reliance on GitHub's hosting availability and consumes action compute minutes (which are free/inexpensive for open source).
