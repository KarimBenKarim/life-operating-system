# ADR 0018: State Management with Zustand and TanStack Query

## Status
Accepted

## Context
Life OS requires an offline-first state management architecture. We have two very different state constraints:
1. **Transient UI State**: Lightweight, high-frequency properties (e.g., sidebar collapse toggles, search modals, active chat pane state, active theme name) that reside entirely in memory and do not need server persistence.
2. **Relational Data Cache**: Highly structured, domain-specific data cached from the local SQLite relational database and local Markdown notes (tasks, goals, habits, biometric timelines, financial ledgers). This state must support optimistic updates, background polling, automatic stale-while-revalidate caches, and immediate refetches triggered by WebSocket synchronization alerts.

## Decision
We select a decoupled state architecture combining **Zustand** (for transient global UI state) and **TanStack Query** (React Query, for relational API data caches):
- **Zustand**: Acts as the single source of truth for UI state, providing tiny, lightweight, boilerplate-free state stores.
- **TanStack Query**: Manages query fetching, automatic caching, garbage collection, optimistic UI updates, and manual refetch invalidations across all REST and SQLite IPC command layers.
- **WebSocket Synchronization Integration**: Global subscription listeners trigger React Query cache invalidations when background Tauri file-watchers update SQLite cache states on disk.

## Rationale
- **Performance**: Standard state libraries (like Redux) introduce massive boilerplate and can trigger app-wide re-renders during high-frequency UI transitions. Zustand uses lightweight React hooks that only re-render components explicitly subscribing to modified slices of state.
- **Cache Management**: TanStack Query handles complex caching logic (retries, deduplication, cache expiry, pagination prefetching, mutation rollbacks) out-of-the-box, saving thousands of lines of custom fetch orchestration code.
- **Offline Reliability**: React Query caches can be persisted locally (using TanStack Query’s local storage persisters), enabling the app to load instantly offline without waiting for database lock initializations.

## Consequences
- **Positive**: Blazing fast rendering speeds, complete decoupling of data synchronization from UI state, and built-in offline cache recovery.
- **Negative**: Requires maintaining two separate store paradigms, but they are conceptually isolated and highly readable.
