# Life OS AI Orchestration Specification

This document specifies the complete **AI Orchestration Architecture** for **Life OS**. It outlines a five-layer agent orchestration system, defines the lifecycle and messaging protocols of personal agents, details planning and tool execution pipelines, specifies recovery and conflict resolution policies, and provides six high-fidelity sequence diagrams.

---

## 1. Five-Layer AI Orchestration Architecture

Life OS separates cognitive agency across five distinct layers. This prevents LLM attention-span saturation, establishes explicit security checkpoints, and guarantees predictable execution loops.

+---------------------------------------------------------------------------------+ | Strategic Layer (Master Life Coach) | +---------------------------------------------------------------------------------+ | v (Long-term Directives) +---------------------------------------------------------------------------------+ | Planning Layer (Weekly Planner) | +---------------------------------------------------------------------------------+ | v (Decomposed Action Plans) +---------------------------------------------------------------------------------+ | Executive Layer (Specialized Domain Agents) | +---------------------------------------------------------------------------------+ | v (Structured Tool Call Requests) +---------------------------------------------------------------------------------+ | Tool Execution Layer (MCP Client Engine) | +---------------------------------------------------------------------------------+ | v (System Executions) +---------------------------------------------------------------------------------+ | Monitoring & Safety Layer (Context Guardrail) | +---------------------------------------------------------------------------------+


### Layer Responsibilities

1. **Strategic Layer (Master Life Coach)**:
   - **Responsibility**: Interprets long-term values, reviews success/failure memories, sets monthly and annual goals, and executes deep quarterly retrospectives.
   - **Interfaces**: Emits monthly/yearly goal vectors; consumes raw journal summaries and year-over-year biometrics.
2. **Planning Layer (Weekly Planner)**:
   - **Responsibility**: Translates long-term goals into structured weekly action plans and daily task backlogs. Schedules recurring habit routines and checks calendar allocations.
   - **Interfaces**: Emits weekly task lists; consumes goals and daily schedule conflict patterns.
3. **Executive Layer (Specialized Domain Agents)**:
   - **Responsibility**: Executes granular sub-tasks within specialized domains. This layer hosts the **Cabinet of Agents** (e.g., Financial Advisor Agent, Sleep/Health Tracker Agent, Note Synthesizer).
   - **Interfaces**: Emits tool execution request schemas; consumes context-masked prompt sessions.
4. **Tool Execution Layer (MCP Client Engine)**:
   - **Responsibility**: Hosts the Model Context Protocol (MCP) clients. Safely maps structured AI tool calls to local system operations (e.g., SQLite insertions, local terminal commands, file alterations).
   - **Interfaces**: Emits raw JSON database and file payloads; consumes tool call configurations.
5. **Monitoring & Safety Layer (Context Guardrail)**:
   - **Responsibility**: Continuous runtime validation. Intercepts all outgoing data streams for context masking (scrubbing secrets) and intercepts all tool executions against strict local safety permissions.
   - **Interfaces**: Emits anonymized token streams to external gateways; raises human-in-the-loop validation blocks.

---

## 2. Agent Lifecycle, Registration, and Communication

### Agent Lifecycle States
An agent in Life OS transitions through four distinct lifecycle states:

[Registering] --> [Active/Idle] <--> [Busy/Executing] --> [Terminated]


1. **Registering**: Agent manifest is validated against the system's JSON schema (schema version, domain namespace boundaries, and requested MCP tool access parameters).
2. **Active/Idle**: Agent resides in-memory, subscribing to its designated event channels and listening for execution tasks.
3. **Busy/Executing**: Agent holds active working memory context and computes a task step.
4. **Terminated**: Resource clean-up. Ephemeral working memory caches are purged immediately.

### Communication Protocol & Message Schema
Agents communicate asynchronously over a local Event Bus using JSON-RPC 2.0 styled messages.

#### Core Message Schema
```json
{
  "jsonrpc": "2.0",
  "id": "msg_90e3ab7d-94c6-43b6-9cc3-f542cc07bc4c",
  "method": "agent.task.execute",
  "params": {
    "sender": "WeeklyPlannerAgent",
    "recipient": "FinancialAdvisorAgent",
    "priority": 100,
    "context_id": "session_881a29cf-a0e2",
    "payload": {
      "task_title": "Compile Weekly Budget Retrospective",
      "time_range": {
        "start": "2026-07-20T00:00:00Z",
        "end": "2026-07-26T23:59:59Z"
      }
    }
  }
}
Event Routing & Discovery
Discovery: Agents query the local AgentRegistryService using specific domain capabilities (e.g., resolve_agent_by_capability("financial_analytics")).
Routing: The registry returns an event namespace subscription pattern (e.g., lifeos.finance.*). The sender broadcasts to this namespace, and the system Event Router handles atomic message delivery.
3. Operations, Planning & Safety Guardrails
Planning & Reflection Cycle
Planning Cycle: Triggered at 07:00 daily. The Planning Agent aggregates active projects, due tasks, calendar events, and yesterday's failure logs to establish the "Today's Focus Note".
Reflection Cycle: Triggered at 23:00 daily. Aggregates completed tasks, logged biometrics (sleep, steps), and sentiment scores. Synthesizes a local Episodic Memory chunk.
Failure Recovery & Conflict Resolution
Failure Recovery Policy: If an executive agent tool call fails (e.g., Bank API timeout), the system attempts a maximum of 3 exponential backoff retries. If failures persist, the transaction is rolled back, the context is saved, and a diagnostic alert is sent to the user's notification ledger.
Conflict Resolution: If two agents attempt concurrent writes (e.g., Sleep Agent logging sleep metric and user editing Daily Journal), the relational ledger lock engine blocks write access, utilizing a pessimistic locking queue in SQLite, with user edits having absolute precedence.
Agent Prioritization:
Priority 1 (CRITICAL): Safety & Guardrail Agents (Context masking, privacy alerts).
Priority 2 (HIGH): Strategic Planning & Active User Tasks (Human-AI interactions).
Priority 3 (LOW): Asynchronous Indexing & Background RAG compilation.
Human-in-the-Loop Approval Workflow
Any tool execution marked as Active/Destructive (e.g., executing a bank transfer, deleting notes, modifying calendar appointments) must generate a persistent approval payload in notifications and pause task progress. It can proceed only upon receiving a cryptographically signed user response.

4. Architectural Sequence Diagrams
Sequence A: User Request Processing
sequenceDiagram
    autonumber
    actor User as User
    participant UI as Front-End Interface
    participant Guard as Safety Layer (Guardrail)
    participant Str as Strategic Layer (Life Coach)
    participant Plan as Planning Layer (Weekly)
    participant Exec as Executive Layer (Finance)

    User->>UI: Types 'Schedule savings goal plan'
    UI->>Guard: Intercept payload
    Guard->>Guard: Context Masking: Scrub sensitive info
    Guard->>Str: Forward clean prompt
    Str->>Plan: Trigger planning adjustments
    Plan->>Exec: Request financial ledger statistics
    Exec-->>Plan: Return ledger margins
    Plan-->>Str: Formulate execution plan
    Str-->>UI: Display drafted savings goal adjustments
    UI-->>User: Visual confirmation prompt
Sequence B: Strategic Planning Loop
sequenceDiagram
    autonumber
    participant Engine as Orchestration Cron
    participant Str as Strategic Layer (Life Coach)
    participant DB as Relational Database (PostgreSQL)
    participant Mem as Semantic Memory (pgvector)

    Engine->>Str: Weekly Retrospective Trigger
    Str->>DB: Query last 7 days of completed goals & habits
    DB-->>Str: Return goal matrices
    Str->>Mem: Query conceptual failure & success memories
    Mem-->>Str: Return similarity matching chunks
    Str->>Str: Synthesize Strategic Guidance Directive (next week)
    Str->>DB: Save guidance markdown note to Note Vault
Sequence C: Task Execution Loop
sequenceDiagram
    autonumber
    participant Plan as Planning Layer
    participant Exec as Executive Domain Agent
    participant MCP as Tool Execution Layer (MCP Client)
    participant Guard as Safety Layer (Guardrail)
    participant System as Local File System (Markdown)

    Plan->>Exec: Delegate task 'Synchronize sleep note'
    Exec->>MCP: Request file modification tool execution
    MCP->>Guard: Check tool permissions
    alt Permitted/Non-Destructive
        Guard->>System: Write updated YAML metadata to SleepNote.md
        System-->>Exec: Return execution success status
    else Restricted/Destructive
        Guard-->>Exec: Throw PermissionBlockException
    end
Sequence D: Feedback Loop & Reflection
sequenceDiagram
    autonumber
    participant Cron as 23:00 Cron Trigger
    participant Exec as Executive Agent (Biometrics)
    participant Mem as Semantic Memory
    participant DB as Relational Database

    Cron->>Exec: Trigger End-of-Day Reflection
    Exec->>DB: Query completed habits & steps count
    DB-->>Exec: Return completed daily logs
    Exec->>Exec: Run regression analysis (Mood vs Sleep)
    Exec->>Mem: Save synthesized correlation (Semantic Memory)
    Mem-->>Cron: End of reflection loop
Sequence E: Daily Planning Cycle
sequenceDiagram
    autonumber
    actor User as User
    participant Plan as Planning Agent
    participant DB as Relational Database
    participant System as Local File System

    User->>Plan: Starts morning session
    Plan->>DB: Fetch today's due tasks & calendar events
    DB-->>Plan: Return schedules
    Plan->>Plan: Order by priority score
    Plan->>System: Create Daily Focus Markdown template
    System-->>User: Display 'Today's Focus Dashboard' (tasks prioritized)
Sequence F: Executive Briefing Generation
sequenceDiagram
    autonumber
    actor User as User
    participant Str as Strategic Agent
    participant Exec as Specialized Cabinet Agents
    participant Guard as Safety Guardrail
    participant UI as Front-End Dashboard

    User->>Str: Request "Morning Executive Briefing"
    Str->>Exec: Gather biometrics, tasks, & finances summaries
    Exec-->>Str: Return structured domain states
    Str->>Str: Synthesize 3-sentence high-impact morning briefing
    Str->>Guard: Validate outbound payload
    Guard->>Guard: Context Masking check (Ensure no raw bank numbers)
    Guard-->>UI: Return scrubbed/safe briefing text
    UI-->>User: Speak/Display Briefing on Home Panel
