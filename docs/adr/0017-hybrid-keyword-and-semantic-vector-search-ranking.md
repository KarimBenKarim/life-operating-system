# ADR 0017: Hybrid Keyword and Semantic Vector Search Ranking

## Status
Accepted

## Context
A key design pillar of Life OS is the "Everything Connects" personal knowledge graph. Users and AI agents need to query notes, journals, and evergreen documents quickly and accurately. Standard search solutions usually rely on either:
1. **Keyword Match (FTS)**: Highly precise for exact filenames, terms, or tags, but blind to synonyms or conceptual relationships.
2. **Semantic Vector Search (pgvector/SQLite-vec)**: Highly capable of understanding conceptual similarities, but poor at matching exact technical codes, dates, or specific names.

Using either system independently leads to missed context or irrelevant results.

## Decision
We select a **Hybrid Search Engine** that combines **PostgreSQL Full-Text Search (FTS)** and **high-dimensional Vector Cosine Similarity (pgvector/SQLite-vec)**, merging results using the **Reciprocal Rank Fusion (RRF)** scoring algorithm:

$$RRF\_Score = \frac{1}{60 + FTS\_Rank} + \frac{1}{60 + Vector\_Rank}$$

- **Keyword Path**: Uses Postgres GIN indexing and BM25 ranking.
- **Vector Path**: Computes 384-dimensional embeddings locally via ONNX runtimes using the `all-MiniLM-L6-v2` model, executing vector searches via pgvector's cosine distance operator (`<=>`).
- **Fusion Layer**: Results are evaluated, ranked by RRF score, and filtered by user/namespace permissions before context assembly.

## Rationale
- **Optimal Retrieval Precision**: Hybrid search yields the highest possible retrieval relevance. It guarantees that if a user searches for an exact term (e.g., "SQL-99 standard"), FTS highlights the exact document, whereas if they search for a concept ("retirement savings"), vector similarity fetches conceptual matches across journals.
- **Explainability**: Since the RRF ranking algorithm is completely deterministic, AI agents can inspect the individual ranks of results, making retrieval steps explainable and predictable.
- **Offline Sovereignty Compliance**: Computing embeddings locally using ONNX runtimes ensures that raw note text is never leaked to external cloud APIs for semantic indexing, preserving complete privacy.

## Consequences
- **Positive**: Exceptional context relevance, full protection of personal data privacy, and robust search capabilities for both users and automated agent RAG pipelines.
- **Negative**: Requires maintaining two indexes (FTS indexes and vector tables), slightly increasing local database storage space.
