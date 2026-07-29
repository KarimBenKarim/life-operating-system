# ADR 0016: Cursor-Based Pagination for Biometric and Financial Logs

## Status
Accepted

## Context
At multi-decade scales, personal operating system datasets will accumulate millions of records, especially in high-frequency domains like biometric sensor streams (heart rates, steps logged every minute) and continuous financial ledger records. Utilizing standard limit-offset pagination (`GET /tasks?offset=10000&limit=50`) over high-frequency streams suffers from two key flaws:
1. **Performance Degradation**: Database engines must scan and skip thousands of index blocks to resolve large offsets, causing performance bottlenecks.
2. **Data Drifting**: If new biometric records or transactions are inserted or deleted while a user is actively paginating, records drift between pages, leading to duplicate or skipped results.

## Decision
We select **Cursor-Based Pagination** for all high-frequency log streams and event lists, while reserving **Offset-Based Pagination** strictly for small, slow-changing tables (such as listing goals or projects).
- **Parameters**: High-frequency APIs accept `starting_after` (a unique resource UUID cursor) and `limit` (page size).
- **Sorting**: Data is strictly ordered chronologically by unique IDs or high-resolution timestamps.
- **Traversal**: To request the next page, clients pass the `next_cursor` UUID returned inside the previous pagination envelope.

## Rationale
- **Sub-Millisecond Queries**: Cursor pagination utilizes strict indexed inequality checks (e.g., `WHERE created_at < cursor_timestamp`), allowing the database to execute fast range scans in $O(1)$ index traversal time regardless of page depth.
- **Consistent Page Boundaries**: Because pagination states are referenced relative to a static record identifier rather than an abstract mathematical index offset, concurrent insertions do not cause data to drift across page views.

## Consequences
- **Positive**: Consistent performance at scale, zero data drifting/missing records during active syncs, and compatibility with continuous scrolling user interfaces.
- **Negative**: Prevents clients from jumping to arbitrary page numbers (e.g., "Page 45"), which is generally not a critical requirement for streaming timelines.
