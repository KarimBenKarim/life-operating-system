# ADR 0025: Immutable System Audit Logging via Cryptographic Hash Chaining

## Status
Accepted

## Context
Because Life OS holds the totality of a person's digital lifecycle, maintaining an absolute, un-repudiable audit trail of system modifications, authentication attempts, and tool executions is essential. If a malicious script or unauthorized user gains temporary access to the local machine, they could perform illicit modifications and subsequently delete or manipulate standard flat audit log records to conceal their tracks.

## Decision
We select an **Immutable Cryptographic Audit Logging** architecture based on row-level hash-chain linking:
1. **Hash Chaining**: Each audit log record contains a `previous_log_hash` column.
2. **Signature Calculation**: The cryptographic signature (`signature`) of the active row is computed using SHA-256 over its parameters combined with the previous row's signature:
   $$Hash_n = SHA256(id_n + timestamp_n + action_n + prev\_hash_{n-1} + after\_state\_hash_n)$$
3. **Startup Verification**: During application bootstrap, the Tauri Local Core runs an asynchronous thread to recalculate and verify the entire log chain. If a signature mismatch is detected (indicating database tampering), the engine locks the system, freezes local directories, and outputs an urgent security warning to the user.

## Rationale
- **Anti-Tampering Integrity**: Hash chaining makes it mathematically impossible to modify or delete a historical audit log entry without breaking the cryptographic signatures of all subsequent entries, allowing the local runtime to immediately detect unauthorized data changes.
- **Explainability**: Chained logs provide a clear, reliable history of *how* and *when* actions occurred, forming a transparent foundation of trust between the user, the core OS, and active AI agents.

## Consequences
- **Positive**: Complete accountability, mathematically-backed data integrity, and immediate tamper detection.
- **Negative**: Adds a minor database insertion compute overhead (computing SHA-256 for a short string takes $<0.05ms$, which is negligible on modern client hardware).
