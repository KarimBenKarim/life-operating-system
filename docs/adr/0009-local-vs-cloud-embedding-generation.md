# ADR 0009: Local vs Cloud Embedding Generation

## Status
Accepted

## Context
Converting text blocks into numerical vector representations (embeddings) is a prerequisites for semantic search. However, sending raw personal notes, deep journals, and intimate diaries to remote cloud embedding APIs (e.g., OpenAI's `text-embedding-3-small`) on every keystroke or file save violates our core data sovereignty and zero-trust privacy tenets.

## Decision
We select **Local Embedding Generation** using the **ONNX Runtime** executing the **all-MiniLM-L6-v2** model directly on the user's client hardware as our mandatory baseline, while permitting cloud-based embeddings purely as a secondary, encrypted, and opt-in toggle.

## Rationale
- **Zero-Leakage Privacy**: By generating embeddings locally, raw notes never leave the user's device for vector index synchronization. The user's entire digital brain remains completely secure.
- **Offline Reliability**: Local embeddings can be generated and queried without internet connectivity, matching our offline-first core architecture requirement.
- **Cost Efficiency**: Generating embeddings for millions of characters is free on local CPUs, sparing the user from cloud subscription or pay-as-you-go API fees.
- **Speed**: Local ONNX execution on all-MiniLM model takes < 2ms per note chunk on consumer-grade hardware, making background indexing completely imperceptible.

## Consequences
- **Positive**: Complete privacy, absolute offline independence, zero operational cost, and zero-latency background indexing.
- **Negative**: The model file must be packaged and shipped inside our desktop installer, adding roughly 90MB to our initial application package download size.
