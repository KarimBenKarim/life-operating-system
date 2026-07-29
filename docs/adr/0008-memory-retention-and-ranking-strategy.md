```markdown
# ADR 0008: Memory Retention and Ranking Strategy

## Status
Accepted

## Context
When a user interacts with Life OS agents, selecting which context chunks to load into the limited context window of an LLM determines response relevance. Relying on basic vector similarity (cosine distance) ignores factors like recency and hard-coded importance (e.g., identity rules), resulting in suboptimal, stale, or bloated context.

## Decision
We select a unified **Weighted Multi-Factor Memory Score** algorithm and a **Perplexity-Based Token Compression** strategy for context ranking and retention:

1. **Ranking Score Formula**:
   $$\text{Memory Score} = (w_1 \cdot \text{Semantic Relevance}) + (w_2 \cdot \text{Temporal Recency}) + (w_3 \cdot \text{Static Importance})$$
   where $w_1 = 0.5$, $w_2 = 0.3$, and $w_3 = 0.2$.
2. **Context Compression**: Embed a lightweight, local, and CPU-efficient token pruning utility (specifically **LLMLingua**) inside the Agentic Gateway.

## Rationale
- **Temporal Relevance**: Human lives drift over time. A note written 5 years ago might have high semantic keyword matches but low present relevance compared to a journal entry written yesterday. Incorporating an exponential decay function ensures recency is prioritized.
- **Cognitive Safety Boundaries**: Static importance weights guarantee that high-priority nodes (such as Identity guidelines and Value bounds) always surface over transient task or log notes regardless of recency or similarity scoring.
- **Context Economy**: Removing redundant syntactic elements using token pruning cuts down token consumption by up to 40%, decreasing public API bills and local context retrieval latency.

## Consequences
- **Positive**: Exceptional prompt relevance, substantial reduction in API bills, and deterministic context selection.
- **Negative**: Adds mathematical and parser computational cycles during the prompt preprocessing phase (adds roughly 15-30ms to prompt dispatch, which is negligible).
