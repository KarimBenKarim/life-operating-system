# ADR 0010: AI Orchestration Layering & Communication Protocol

## Status
Accepted

## Context
Designing a multi-agent system requires establishing how agents discover, coordinate, and communicate with one another. Allowing tight, synchronous connections between agents or letting them directly invoke each other's execution contexts creates rigid, unmaintainable dependency graphs and increases execution blocking.

## Decision
We select an **Asynchronous Event-Driven Bus** utilizing a **5-Layer Cognitive Architecture** and **JSON-RPC 2.0 Styled Message Schemas**:

1. **The 5-Layer Stack**: Explicit separation between Strategic, Planning, Executive, Tool Execution, and Monitoring & Safety Layers.
2. **Asynchronous Messaging**: Agents communicate exclusively by publishing and subscribing to namespaces (e.g., `lifeos.finance.*`) on a local Event Bus.
3. **Structured Schemas**: All interactions must adhere strictly to JSON-RPC 2.0 formatting, identifying sender, recipient, context ID, and method namespaces.

## Rationale
- **Decoupling**: Decoupling agents ensures that individual agents can go offline, crash, or be updated without causing cascading runtime failure in other domains.
- **Explainability & Auditing**: Since all agent-to-agent interactions flow as structured event packets over a central bus, the system can write a complete, structured log of *how* an agent made a decision, forming a clear logical path for the user to review.
- **Attention Management**: Multi-tiered layers prevent executive agents from needing to process long-term strategic plans, saving context tokens and optimizing cognitive focus.

## Consequences
- **Positive**: Complete loose coupling, stellar operational observability, and simplified unit-testing of isolated agents.
- **Negative**: Adds event queue and message deserialization processing overhead (unnoticeable on desktop hardware, roughly < 1ms per delivery cycle).
