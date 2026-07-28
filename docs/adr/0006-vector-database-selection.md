# ADR 0006: Vector Database Selection

## Status
Accepted

## Context
Life OS requires low-latency retrieval of semantic context (such as raw notes, research logs, journals, and conversation fragments) to pass high-quality context to local and remote AI models. We need a performant vector database solution to store and query high-dimensional floating-point embeddings.

## Decision
We select **pgvector** (PostgreSQL Extension) as the primary vector storage engine, while maintaining compatibility with **SQLite-vec** for local desktop environments.

## Rationale
- **Infrastructure Consolidation**: Reusing PostgreSQL via `pgvector` eliminates the architectural complexity, server management, and network lag of introducing separate dedicated vector search engines like Pinecone, Milvus, or Qdrant.
- **Relational Cohesion**: It allows us to perform single-query hybrid search. We can combine semantic similarity scoring with strict SQL relational filters (e.g., `WHERE document.user_id = ? AND document.created_at >= ?`) in a single index-backed join, which dedicated vector databases cannot achieve cleanly.
- **Indices Support**: pgvector natively supports HNSW (Hierarchical Navigable Small World) and IVFFlat indices, ensuring sub-10ms retrieval times at scale.

## Consequences
- **Positive**: Simplified microservices layout, zero additional server hosting cost, unified transactional safety, and robust hybrid search capabilities.
- **Negative**: Increases PostgreSQL CPU load during high-frequency index rebuilds, which we mitigate by generating embeddings asynchronously.
