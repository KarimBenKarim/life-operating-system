# ADR 0023: Privacy-Preserving Context Masking Gateway for External LLMs

## Status
Accepted

## Context
When utilizing advanced, remote cloud LLM endpoints (such as OpenAI or Anthropic) for RAG summaries, financial correlation reports, or active conversational agents, sending raw notes containing highly sensitive context (e.g., credit card numbers, bank balances, SSNs, or addresses) violates user privacy guidelines. We need a way to leverage powerful cloud models without exposing raw, high-entropy personal identifiers to external API servers.

## Decision
We select an automated local-first **Context Masking Gateway** inside the Local Engine Core:
1. **Local Interception**: Before any prompt is dispatched to an external API, the Local Engine runs regex and Named-Entity Recognition (NER) patterns over the text context locally on the host device.
2. **Token Replacement**: Personal identification strings, account numbers, and balances are scrubbed and replaced with anonymous placeholder tokens (e.g., `$2,435.12` becomes `[BALANCE_VAL_1]`).
3. **Translation Map**: A temporary mapping table is preserved inside the local transient memory cache.
4. **Response Re-hydration**: When the response is received from the remote LLM, the local gateway reverses the mapping to present the correct context seamlessly to the user inside the React UI.

## Rationale
- **Zero Privacy Leakage**: External cloud providers only receive low-entropy, scrubbed metadata, making it impossible for them to build identifiable profiles or train models on the user's private data.
- **Resource Preservation**: Running lightweight regex/NER patterns locally on the client's host CPU consumes negligible resources ($<5ms$ execution overhead) while preventing massive security leaks.

## Consequences
- **Positive**: Complete privacy, absolute zero data leakage to public AI training datasets, and support for using highly-capable external models.
- **Negative**: Adds a processing checkpoint inside prompt construction pipelines, and some highly specialized text patterns may occasionally pass through if they bypass the regex/NER boundaries.
