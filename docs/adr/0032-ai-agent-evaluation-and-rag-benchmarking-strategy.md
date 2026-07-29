# ADR 0032: AI Agent Evaluation and RAG Benchmarking Strategy

## Status
Accepted

## Context
Standard deterministic test assertions (such as check-equal) are completely ineffective for evaluating conversational AI agent outputs, as LLM outputs are naturally non-deterministic and can vary on every run. If an agent persona exhibits conversational drift, starts hallucinating sleep or transaction metrics, or succumbs to indirect prompt injections, flat code unit tests are blind to these failures.

## Decision
We select an automated **LLM-as-a-Judge Evaluation Pipeline** utilizing the **Ragas Evaluation Framework** inside our continuous integration builds:
1. **Automated Judging**: Every major pull request executes a sandboxed evaluation suite. An isolated evaluation judge model (like GPT-4o or local LLaMA-3 evaluation models) scores agent responses across four specific parameters:
   - **Faithfulness** (relevance to grounded local context).
   - **Answer Relevance** (directness to user prompts).
   - **Context Recall** (precision of local RAG note retrieval).
   - **Prompt Injection Resistance** (resilience to embedded user commands).
2. **Build Failure Threshold**: The CI pipeline enforces that the average Faithfulness and Injection Resistance scores must stay strictly at or above **`0.90`**. If a score drops, the merge build is forcefully aborted.

## Rationale
- **Continuous Quality Control**: Automated benchmarks allow developers to instantly check if modifying a system instruction prompt, fine-tuning a model, or updating the retrieval chunking algorithm has inadvertently caused conversational regressions or safety holes.
- **Explainability**: The evaluation parameters output detailed, quantifiable scores, making AI behavior completely transparent and auditable for users and self-hosters.

## Consequences
- **Positive**: Absolute protection against AI hallucinations, automated resistance against indirect prompt injections, and robust performance analytics over multi-agent reasoning.
- **Negative**: Adds minor cost and time overhead to CI/CD pipeline runs (running evaluations requires dedicated API credits and can take up to 2-3 minutes).
