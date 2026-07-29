# ADR 0011: Human-in-the-Loop Tool Execution & Permissions

## Status
Accepted

## Context
AI agents are capable of executing system tasks via MCP tools, such as deleting notes, moving financial records, or altering scheduled calendar events. Granting autonomous, unchecked write access to agents on a user's local disk can lead to accidental data loss, corrupt accounting records, or schedule collisions.

## Decision
We select a strict **Human-in-the-Loop (HITL) Validation** architecture and **Granular Tool Permission Profiles**:

1. **Destructive vs. Non-Destructive**: All MCP tools are classified statically:
   - **Non-Destructive** (e.g., `read_notes`, `search_index`): Can execute autonomously without human approval.
   - **Active/Destructive** (e.g., `delete_note`, `update_calendar`, `write_ledger`): Automatically halted by the Safety & Guardrail Layer.
2. **Approval Action**: Active/Destructive actions generate a pending notification prompt. The agent pauses execution and waits for explicit human sign-off via the interface.

## Rationale
- **User Agency**: The user retains complete control over their physical environment. AI agents serve as assistants rather than autonomous actors.
- **System Stability**: Restricting automated write operations ensures that database schemas and vault files cannot be corrupted by hallucinations or formatting bugs.
- **Trust Alignment**: Transparent consent loops build long-term, low-friction collaboration between the user and the system.

## Consequences
- **Positive**: Complete safety, zero risk of autonomous file deletions, and transparent audit trails for all actions.
- **Negative**: Adds a friction checkpoint to agentic workflows, requiring users to explicitly confirm changes before they occur.
