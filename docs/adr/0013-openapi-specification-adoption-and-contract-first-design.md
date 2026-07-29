# ADR 0013: OpenAPI Specification Adoption & Contract-First Design

## Status
Accepted

## Context
As the Life OS ecosystem expands, several frontend clients (Tauri desktop app, native mobile capacitor wrappers, third-party companion widgets) and external developers will need to query the gateway. Writing APIs without strict, machine-readable contracts leads to structural drift, outdated client SDKs, poor type-safety boundaries, and validation errors.

## Decision
We select **OpenAPI Specification (OAS) 3.0** as our mandatory interface description standard. We adopt a **Contract-First Design** methodology: any API route or schema change must first be proposed and updated inside the OpenAPI spec document before any server-side route logic is modified.

## Rationale
- **Strong Compile-Time Type-Safety**: Frontend apps can utilize automated code generation engines (e.g., `openapi-typescript` or `orval`) to generate fully-typed API clients directly from the spec, completely eliminating networking type drift.
- **Auto-Generated Interactive Documentation**: Developers and self-hosters immediately receive interactive, localized documentation (via Swagger UI or Scalar) out of the box.
- **Contract-Based Testing**: Enables the use of automated contract testing tools to verify that incoming payloads and outgoing server mock responses exactly match the documented interface.

## Consequences
- **Positive**: Eradicates networking boundary type bugs, provides instant interactive documentation, and streamlines cross-team collaboration.
- **Negative**: Adds overhead during initial design phases as all schema updates must first be modeled inside the YAML spec files.
