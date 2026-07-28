
# ADR 0004: AI Agent Integration via Model Context Protocol (MCP)

## Status
Accepted

## Context
Life OS is designed from the ground up as an interactive, agent-native workspace. AI agents must be able to retrieve note context, update task lists, log habit metrics, and run financial pattern queries. However, creating tight, proprietary bindings between client interfaces and specific AI vendors results in poor extensibility and vendor lock-in.

## Decision
We choose the **Model Context Protocol (MCP)** as the standard communication layer between specialized AI agents and the Life OS core system.

### Integration Details
1. **Local MCP Server**: Life OS runs an embedded local MCP server.
2. **Exposed Tools**: The MCP server exposes granular, safe, and read/write-controlled tools to local and remote LLM orchestrators:
   - `search_notes`: Full-text note querying.
   - `get_biometric_patterns`: Summarized sleep, mood, and activity correlation query tool.
   - `update_task_status`: Safely update task completion states.
   - `draft_journal_entry`: Append formatted text drafts to files inside the Markdown Vault.
3. **Local Models Default**: The agent gateway prioritizes routing agent prompts to local inference engines (such as **Ollama**) executing LLaMA 3, Mistral, or Qwen models.

## Rationale
- **Interoperability**: MCP is a standardized protocol. It allows the user to bring their own agent client (e.g., Claude Desktop, Zed, or Claude Code) and seamlessly connect it directly to their Life OS vault.
- **Safety through Permissioning**: The user can configure granular access policies. For example, the "Sleep Coach Agent" can be restricted to only access `get_biometric_patterns` and `search_notes` with `#sleep` hashtags, completely blinding it from financial ledger tools.
- **Modularity**: Specialized agent personas (such as Habit Agent or Finance Tracker) are isolated behind standard interfaces, allowing them to be developed, swapped, or updated independently.

## Consequences
- **Positive**: Standardized agent integration, robust security boundaries, extreme modularity, and plug-and-play support for next-generation developer tooling.
- **Negative**: Adds protocol layer complexity (JSON-RPC 2.0 communication over STDIO/SSE) requiring robust error handling and stream monitoring.
