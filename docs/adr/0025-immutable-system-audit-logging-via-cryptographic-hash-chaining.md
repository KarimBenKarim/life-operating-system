# ADR 0025: Immutable System Audit Logging via Cryptographic Hash Chaining

## Status
Accepted (Updated)

## Context
Because Life OS holds the totality of a person's digital lifecycle, maintaining an absolute, un-repudiable audit trail of system modifications, authentication attempts, and tool executions is essential. If a malicious script or unauthorized user gains temporary access to the local machine, they could perform illicit modifications and subsequently delete or manipulate standard flat audit log records to conceal their tracks.

## Decision
We select an **Immutable Cryptographic Audit Logging** architecture based on row-level hash-chain linking:
1. **Hash Chaining & Length-Prefixed Canonicalization**: Each audit log record contains a `previous_hash` column. The cryptographic signature (`hash`) of row $n$ is computed using SHA-256 over deterministic length-prefixed string representations of all fields:
   $$Hash_n = SHA256(\text{encode\_field}(id_n) + \text{encode\_field}(created\_at_n) + \text{encode\_field}(user\_id_n) + \text{encode\_field}(action\_type_n) + \dots + \text{encode\_field}(prev\_hash_{n-1}))$$
   where $\text{encode\_field}(v) = \text{length}(v) + \text{":"} + v + \text{";"}$.
   This length-prefixed format guarantees boundary unambiguous hashing even when field values contain special characters or delimiters.
2. **Atomic Write Serialization**: Audit log insertion executes inside an immediate transaction (`TransactionBehavior::Immediate`), acquiring an immediate write lock on the database to prevent write races and ensure atomic sequence and hash generation across concurrent writers.
3. **Startup Verification**: During application bootstrap, `initialize_and_verify_database` executes pending migrations and recalculates/verifies the entire audit chain before returning a usable database connection.
4. **Verification Guarantees & Known Limitations**:
   - **Guaranteed Detection**: Modifying any field in an existing record, deleting a record in the first or middle positions, inserting a forged record, or reordering rows invalidates the SHA-256 chain and is detected during verification.
   - **Tail Truncation Limitation**: A pure internal hash chain cannot detect the deletion of the trailing (tail) entries without an external head/count checkpoint, as the remaining prefix chain 1..N-1 remains internally self-consistent. External head checkpoint tracking is specified as a system-level runtime orchestration follow-up.
   - **System Response**: System-wide OS lockdown and local directory freezing upon tamper detection are specified as system-level runtime orchestration follow-ups.

## Rationale
- **Anti-Tampering Integrity**: Hash chaining makes it mathematically impossible to modify or delete a historical audit log entry without breaking the cryptographic signatures of all subsequent entries, allowing the local runtime to immediately detect unauthorized data changes.
- **Unambiguous Canonical Encoding**: Length-prefixed encoding prevents canonicalization collisions between adjacent fields regardless of string contents.
- **Explainability**: Chained logs provide a clear, reliable history of *how* and *when* actions occurred, forming a transparent foundation of trust between the user, the core OS, and active AI agents.

## Consequences
- **Positive**: Complete accountability, mathematically-backed data integrity, deterministic canonical hashing, and immediate tamper detection.
- **Negative**: Adds a minor database insertion compute overhead (computing SHA-256 for a short string takes $<0.05ms$, which is negligible on modern client hardware).
