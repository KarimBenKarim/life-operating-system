# ADR 0015: RFC 7807 and Custom Validation Errors for Robust API Feedback

## Status
Accepted

## Context
When developers, frontend frameworks, or autonomous AI agents query the Life OS Gateway, they must receive precise, structured feedback when things go wrong (e.g., input schema violations, resource lock collisions, or permission blocks). Using generic or inconsistent HTTP status codes and error payloads leads to brittle parsing, poor UI error reporting, and makes it difficult for AI agents to self-correct during failed tool calls.

## Decision
We mandate **RFC 7807 Problem Details** as the universal error-reporting schema for all REST-based API responses. All error responses must utilize the `application/problem+json` media type and contain the following fields:
- `type`: A URI reference identifying the specific error category (e.g., `https://api.lifeos.org/errors/validation-failed`).
- `title`: A short, human-readable summary of the problem.
- `status`: The exact HTTP status code duplicated for clear JSON-level parsing.
- `detail`: A detailed, context-aware explanation of the occurrence.
- `instance`: A URI reference indicating the specific resource instance or endpoint where the error occurred.
- `errors` (optional array): Granular validation failure details specifying the failing input `field` and custom constraint `message`.

## Rationale
- **Standardization**: Adhering to RFC 7807 guarantees that clients can construct a single, standardized error-handling middleware instead of parsing diverse JSON schemas.
- **AI Agent Self-Correction**: Because the error responses specify *why* an action failed in structured form (e.g., indicating that `due_date` must be greater than today), AI agents executing MCP tool commands can catch these payloads, parse the validation requirements, and automatically reformulate and retry their requests.
- **Rich Form Validation**: Passing an array of field-level constraints allows frontend client libraries (such as Formik or React Hook Form) to immediately map validation errors back to specific UI input blocks.

## Consequences
- **Positive**: Machine-readable error handling, seamless forms alignment, and enhanced agentic self-healing capabilities.
- **Negative**: Adds minor serialization overhead on the Gateway, requiring developers to write custom exception filters to map internal code errors to the RFC 7807 structure.
