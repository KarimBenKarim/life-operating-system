# ADR 0012: Hybrid REST & WebSocket Protocol Selection

## Status
Accepted

## Context
Life OS has two competing runtime API constraints:
1. **Transactional Safety & Simplicity**: Standard CRUD modifications (e.g., establishing goals, creating tasks, uploading files) require predictable, isolated, stateless requests that align with standard REST practices.
2. **Real-time Live Streams**: Background file watchers, live biometrics streaming, active chat rooms, and background agent logs require persistent, bidirectional, and highly efficient network delivery without polling.

## Decision
We select a **Hybrid API Gateway Architecture** combining **RESTful HTTPS endpoints** (for transactional state mutations and file uploads) with **WebSocket (WSS) Pub/Sub connections** (for real-time event routing and background worker logs).

## Rationale
- **REST for Transactions**: REST provides standard HTTP caching, clean authorization headers, safe status code mapping (RFC 7807 problem details), and simpler integration with existing developer utilities.
- **WebSocket for Duplex Streams**: Running high-frequency biometric streams or chat interfaces over HTTP polling causes extreme database wear and massive network packet overhead. WebSockets establish a single TCP handshake, allowing immediate server-to-client frame pushes with minimal bytes of frame overhead.
- **Separation of Concerns**: Transaction logic and live event streams are strictly isolated, making our microservices easier to scale and maintain separately.

## Consequences
- **Positive**: Sub-millisecond live updates on UI dashboards, reliable transactional CRUD boundaries, and optimal use of server computing cycles.
- **Negative**: Adds configuration complexity to our API gateway, which must now support secure proxying of both standard HTTP and persistent WebSocket frames.
