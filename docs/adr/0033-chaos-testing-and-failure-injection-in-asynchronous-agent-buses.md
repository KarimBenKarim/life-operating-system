# ADR 0033: Chaos Testing and Failure Injection in Asynchronous Agent Buses

## Status
Accepted

## Context
As an offline-first personal operating system, Life OS must remain completely resilient and responsive under volatile execution constraints (e.g., sudden physical network dropouts during active syncs, local database lock contentions, background daemon timeouts, or Redis session container crashes). If the client application crashes, locks up, or corrupts local markdown files during an unexpected connection loss, user trust and durability guarantees are broken.

## Decision
We select an automated **Chaos Engineering & Failure Injection Strategy**:
1. **Automated Fault Injection**: We write dedicated chaos validation scripts within our integration/E2E test suite. These scripts programmatically trigger unexpected failures during active, high-traffic user transactions:
   - Abruptly killing the Postgres/Redis containers.
   - Injecting artificial network packet delays ($>2000\text{ms}$) and simulated packet loss.
   - Locking local SQLite threads during active write-synchronizations.
2. **Resilience Assertions**: We assert that the local client must cleanly catch these faults, display non-disruptive offline status alerts, queue write mutations inside local SQLite, and trigger retries using exponential backoffs, all without dropping frames or corrupting Markdown files.

## Rationale
- **Guaranteed Durability**: Injecting errors programmatically is the only way to mathematically verify that our transactional boundaries, database rollback handlers, and local synchronization layers are 100% robust.
- **Improved User Experience**: Proactively validating failure states ensures that the user interface never freezes, freezes up, or crashes when a user enters an elevator, loses Wi-Fi, or experiences background database locks.

## Consequences
- **Positive**: Complete local database integrity under extreme environments, seamless offline-to-online recovery, and self-healing systems.
- **Negative**: Adds configuration and setup complexity to our integration tests, requiring custom container orchestrators to simulate localized service drops.
