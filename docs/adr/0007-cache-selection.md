# ADR 0007: Cache Selection

## Status
Accepted

## Context
Life OS features real-time, interactive user interfaces like habit-heatmaps, calendar dashboards, and AI agent panels. Computing complex analytics (e.g., multi-year streak logs, financial aggregations, and session-based prompt context masking tables) on every single page load can exhaust database resources and degrade client response rates. We require an extremely fast cache layer.

## Decision
We select **Redis** as the primary caching and session store for Life OS server deployments, while utilizing a lightweight **local in-memory LRU cache** inside desktop client builds.

## Rationale
- **Sub-Millisecond Read Latency**: Redis stores data fully in-memory, ensuring extremely rapid reads of pre-calculated telemetry matrices.
- **Advanced Data Structures**: Support for Hashes, Sets, and Sorted Sets simplifies implementation of rate limiters, session managers, and real-time active user telemetry.
- **Pub/Sub capabilities**: Redis Pub/Sub offers a clean mechanism to broadcast system file-watcher synchronization notifications directly to connected Web UI clients without polling.

## Consequences
- **Positive**: Blazing fast dashboards, lower relational database pressure, and integrated real-time pub/sub synchronization capabilities.
- **Negative**: Adds an additional dependency to the server deployment stack, which self-hosters must manage. We mitigate this by making Redis optional, defaulting to in-memory processes if no Redis instance is configured.
