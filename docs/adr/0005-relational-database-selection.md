```markdown
# ADR 0005: Relational Database Selection

## Status
Accepted

## Context
Life OS manages structured datasets including users, identities, task trees, goal structures, habit parameters, calendar bookings, audit trails, and multi-layered relationships. To ensure total consistency, sub-millisecond response rates, and absolute transactional safety, we require a highly robust relational engine.

## Decision
We select **PostgreSQL** as the primary relational database for the Life OS server architecture, while continuing to support **SQLite** inside local desktop Tauri wrappers as a local-first cache/index container.

## Rationale
- **Constraint Enforcement**: PostgreSQL offers advanced constraint management (e.g., partial indexes, cross-table checks, and domain checks) ensuring normalization rules (3NF) are kept without data corruption.
- **Enterprise Capabilities**: Provides native table partitioning (crucial for long-term telemetry like `habit_logs` and `system_audit_logs`), native UUIDv4 generators, JSONB support for unstructured API payloads (e.g., identity oauth tokens), and rich full-text search.
- **Data Sovereignty Ecosystem**: Extensive open-source support ensures that users who self-host Life OS can easily spin up a standard Postgres container without licensing friction.

## Consequences
- **Positive**: Complete ACID safety, highly efficient scale-out via read-replicas, and native integration with extension tools such as `pgvector`.
- **Negative**: Higher memory and installation overhead compared to embedded SQLite, which is why we retain SQLite as our local/offline fallback wrapper.
