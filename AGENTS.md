# AGENTS.md

## Project

Life Operating System

## Goal

Build a long-term AI-powered operating system for managing every aspect of a person's life.

## General Rules

- Always preserve Clean Architecture.
- Keep modules loosely coupled.
- Prefer composition over inheritance.
- Every module must be independently testable.
- Write documentation alongside code.
- Never introduce dependencies without justification.
- Prefer async Python where appropriate.
- Follow SOLID principles.
- Every Pull Request must pass tests.

## Architecture

The system is organised into five AI layers.

See docs/vision.md.

## Documentation

Before implementing major features:

- update architecture documentation
- update Mermaid diagrams
- update ADRs

## Testing

pytest

Coverage >90%

## Formatting

ruff

black

mypy
