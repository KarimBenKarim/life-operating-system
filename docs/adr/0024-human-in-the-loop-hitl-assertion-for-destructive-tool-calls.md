# ADR 0024: Human-in-the-Loop HITL Assertion for Destructive Tool Calls

## Status
Accepted

## Context
AI agents operating inside Life OS utilize the Model Context Protocol (MCP) to execute system-level actions such as creating, updating, or deleting notes, database items, tasks, and scheduling items. Granting autonomous, unchecked write/delete access to autonomous LLMs poses severe stability risks, including accidental data loss, schema corruption due to hallucinations, or schedule collisions.

## Decision
We select a strict **Human-in-the-Loop (HITL) Interception Gateway**:
1. **Tool Classification**: All system-exposing MCP tools are statically classified:
   - **Non-Destructive** (e.g., `search_notes`, `get_biometric_patterns`): Allowed to run autonomously without intervention.
   - **Destructive** (e.g., `update_task_status` with status = archived, `delete_note`): Intercepted immediately by the Local Safety Layer.
2. **Execution Block**: When a destructive tool is triggered, the Local Engine halts the execution thread, generates a pending notification token, and raises an approval modal on the user's dashboard screen.
3. **Explicit User Signature**: The tool call can proceed only when the user explicitly clicks "Approve" (issuing a signed POST request), releasing the SQLite/FS thread lock.

## Rationale
- **User Sovereignty**: The user remains the definitive, absolute authority over their digital environment. No AI agent can mutate or delete system data without human consent.
- **Accident Prevention**: Intercepting destructive commands eliminates the risk of system-wide file deletions or database corruption caused by LLM errors or prompt injections.

## Consequences
- **Positive**: Exceptional safety boundaries, zero risk of autonomous file deletions, and complete audit histories for all executed tool steps.
- **Negative**: Adds a minor friction point to conversational AI agent workflows, requiring explicit clicks before files are modified.
