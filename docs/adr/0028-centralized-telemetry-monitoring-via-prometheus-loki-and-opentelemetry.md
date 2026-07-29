# ADR 0028: Centralized Telemetry Monitoring via Prometheus, Loki, and OpenTelemetry

## Status
Accepted

## Context
In a multi-agent system, tracing conversational paths, database read/writes, and tool executions is highly complex. If an agent fails to write a ledger transaction or experiences high latency during RAG context assembly, standard monolithic logging makes it nearly impossible to reconstruct the chronological thread of actions. We need a modern, integrated observability stack to track metrics, logs, and distributed traces.

## Decision
We select an integrated observability stack combining **Prometheus** (for system metrics), **Grafana Loki** (for centralized logs), and **OpenTelemetry / Jaeger** (for distributed tracing):
1. **Metrics (Prometheus)**: System containers expose a `/metrics` endpoint. Prometheus scrapes these on a 15-second interval, tracking performance histograms (database connection pool size, CPU/memory usage, query durations).
2. **Logs (Loki)**: Promtail gathers stdout/stderr logs from all containers and ships them as structured JSON strings to Loki.
3. **Tracing (OpenTelemetry)**: Every multi-agent event or tool execution is wrapped inside an OpenTelemetry context trace. The `context_id` (trace ID) is propagated across JSON-RPC event bus headers, allowing Jaeger to stitch together waterfall execution diagrams of agent reasoning chains.

## Rationale
- **Trace Visualization**: Distributed tracing allows developers to visually isolate exactly where an agentic task bottleneck or failure occurred (e.g., separating LLM response latency from database query delay).
- **Integrated Dashboards**: Grafana aggregates Prometheus metrics, Loki log blocks, and Jaeger tracing charts into a unified dashboard view, drastically accelerating debug times.
- **Resource Efficiency**: Loki index-free log storage is extremely fast and consumes a fraction of the memory and storage required by heavy Elasticsearch stacks.

## Consequences
- **Positive**: Exceptional operational observability, horizontal trace alignment across asynchronous micro-agent queues, and automated alerting thresholds.
- **Negative**: Adds minor tracing instrument code into backend gateways and increases local hosting requirements by roughly $250\text{MB}$ of memory for the telemetry daemons.
