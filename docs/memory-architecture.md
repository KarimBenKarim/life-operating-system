# Life OS AI Memory Architecture

This document specifies the complete **AI Memory Architecture** for **Life OS**. It outlines a multi-tiered memory taxonomy spanning 16 specialized memory types, details the memory ingestion and retrieval pipelines, specifies context assembly and compression mechanics, and provides four high-fidelity Mermaid architectural diagrams.

---

## 1. Memory Architecture & Core Taxonomy (16 Memory Types)

To replicate the cognitive processing of a human brain, Life OS organizes memory into 16 distinct subsystems across three main temporal categories: **Strategic Long-Term Memory**, **Contextual/Domain Memory**, and **Subjective Episodic & Working Memory**.

                       +-----------------------------------+
                       |            Working Memory         |
                       |       (Transient LLM Context)     |
                       +-----------------------------------+
                                         ^
                                         | (Assembles Context)
                                         |
        +--------------------------------+--------------------------------+
        |                                |                                |
+-----------------------+ +-----------------------+ +-----------------------+ | Strategic Long-Term | | Contextual Domain | | Subjective Episodic | | Memory | | Memory | | & Event Memory | +-----------------------+ +-----------------------+ +-----------------------+ | - Identity Memory | | - Goal Memory | | - Episodic Memory | | - Values Memory | | - Project Memory | | - Semantic Memory | | - Decision Memory | | - Task Memory | | | | - Success Memory | | - Research Memory | | | | - Failure Memory | | - Knowledge Memory | | | | | | - Health Memory | | | | | | - Finance Memory | | | | | | - Relationship Memory | | | +-----------------------+ +-----------------------+ +-----------------------+


---

### Group A: Strategic Long-Term Memory

#### 1. Identity Memory
- **Purpose**: Defines the user's core attributes, personality profiles, and preferred communication style.
- **Stored Information**: Name, date of birth, bio, MBTI-like profiles, communication preferences, and local user profiles.
- **Relationships**: Parent of all user-centric memories; links closely with Working Memory.
- **Update Policy**: Manual update by the user, or automated slow drift compilation based on quarterly reviews.
- **Retrieval Strategy**: Injected statically into every system-level AI prompt gateway.
- **Expiration Policy**: Permanent.
- **Privacy Considerations**: Classified as High Sensitivity. Must never be sent to third-party public AI models without local pseudonymization.

#### 2. Values Memory
- **Purpose**: Guides the AI agent on the moral, philosophical, and personal boundaries of the user.
- **Stored Information**: Core personal values, philosophical tenets, boundaries, and life rules.
- **Relationships**: Integrates with Goal Memory to evaluate if goals align with personal values.
- **Update Policy**: Updated during annual retrospective reviews or manually via a dedicated values note.
- **Retrieval Strategy**: Retrieved whenever the user establishes a new Goal or Project.
- **Expiration Policy**: Permanent.
- **Privacy Considerations**: Highly sensitive. Fully processed locally.

#### 3. Decision Memory
- **Purpose**: Records past significant decisions, the context in which they were made, and their subsequent outcomes.
- **Stored Information**: Dilemma, constraints, chosen option, alternative options considered, and outcomes.
- **Relationships**: Connected to Note, Project, and Journal aggregates.
- **Update Policy**: Captured during Monthly and Quarterly Reviews.
- **Retrieval Strategy**: Vector lookup during the creation of new high-impact Projects.
- **Expiration Policy**: Indefinite.
- **Privacy Considerations**: Moderate Sensitivity.

#### 4. Success Memory
- **Purpose**: Tracks past achievements to help AI agents reinforce positive behavior and extract successful patterns.
- **Stored Information**: Milestone title, metric target reached, date completed, habits that contributed, and retrospective notes.
- **Relationships**: Direct edge connection with Goals and Habits.
- **Update Policy**: Automated upon Goal completion or positive biometric telemetry streak achievements.
- **Retrieval Strategy**: Retrospective prompts for quarterly planning.
- **Expiration Policy**: Permanent.
- **Privacy Considerations**: Low Sensitivity.

#### 5. Failure Memory
- **Purpose**: Captures setbacks, failed habits, and unmet goals as learning context to avoid recurring patterns.
- **Stored Information**: Goal/Habit abandoned, trigger, retrospective analysis, and systemic adjustments proposed.
- **Relationships**: Linked to corresponding abandoned Habits and Projects.
- **Update Policy**: Automated on Goal/Habit cancellation or manual logging in reflection templates.
- **Retrieval Strategy**: Injected during Weekly planning sessions to suggest modifications to habit triggers.
- **Expiration Policy**: Indefinite.
- **Privacy Considerations**: High Sensitivity. Requires local summarization before external prompt formatting.

---

### Group B: Contextual Domain Memory

#### 6. Goal Memory
- **Purpose**: Maintains current and past milestones and active directions of the user.
- **Stored Information**: Goal Title, Description, KPI metric targets, and linked Projects.
- **Relationships**: One-to-many relationship with Projects; linked to Success and Failure memory.
- **Update Policy**: Real-time write/read synced with PostgreSQL `goals` table.
- **Retrieval Strategy**: Priority retrieval for strategic agents.
- **Expiration Policy**: Moves to archive state once completed.
- **Privacy Considerations**: Low Sensitivity.

#### 7. Project Memory
- **Purpose**: Manages active outcome-driven milestones requiring multiple tasks.
- **Stored Information**: Project scope, target date, progress state, and notes.
- **Relationships**: Linked to a parent Goal and multiple child Tasks.
- **Update Policy**: Real-time transactional write in PostgreSQL.
- **Retrieval Strategy**: Injected during Daily and Weekly dashboard updates.
- **Expiration Policy**: Completed projects are archived after 90 days.
- **Privacy Considerations**: Low to Moderate Sensitivity.

#### 8. Task Memory
- **Purpose**: Keeps trace of micro-actions, priorities, and daily schedules.
- **Stored Information**: Task title, status, priority, due date, and recurrence rules.
- **Relationships**: Connected to parent Project.
- **Update Policy**: Real-time write-through via API or local Markdown parser.
- **Retrieval Strategy**: High frequency SQL indices lookup.
- **Expiration Policy**: Purged or archived 30 days after completion.
- **Privacy Considerations**: Low Sensitivity.

#### 9. Research Memory
- **Purpose**: Stores active investigations, bookmark analyses, and topical studies.
- **Stored Information**: Raw articles parsed, source URLs, highlights, and annotations.
- **Relationships**: References Note nodes.
- **Update Policy**: Triggered on clip/capture events.
- **Retrieval Strategy**: Semantic search query.
- **Expiration Policy**: Soft-archived after 180 days of inactivity.
- **Privacy Considerations**: Low Sensitivity.

#### 10. Knowledge Memory
- **Purpose**: Houses evergreen wiki documents, instructions, guides, recipes, and notes (the "Second Brain").
- **Stored Information**: Note contents, cross-links, tags, and structured headers.
- **Relationships**: Rich note relationships with backlink edges.
- **Update Policy**: Synchronized on Markdown Note Vault file changes.
- **Retrieval Strategy**: Hybrid search (BM25 + pgvector cosine similarity).
- **Expiration Policy**: Permanent.
- **Privacy Considerations**: Moderate to High Sensitivity. Local-only embedding indexing.

#### 11. Health Memory
- **Purpose**: Tracks biometrics, activity, and sleep patterns over time.
- **Stored Information**: Daily steps, calories, sleep quality metrics, and heart rate trends.
- **Relationships**: Connected to Daily Journal entries.
- **Update Policy**: Daily batch ingestion from external APIs (Apple Health, Fitbit).
- **Retrieval Strategy**: Injected into weekly health correlation services.
- **Expiration Policy**: Retained permanently for long-term health trends.
- **Privacy Considerations**: High Sensitivity. Must remain strictly local. Only abstract statistical outputs are allowed to leave local limits.

#### 12. Finance Memory
- **Purpose**: Tracks accounts, ledgers, transactions, and categories.
- **Stored Information**: Amount, date, payee, category, and account balances.
- **Relationships**: Links to projects and habits (for tracking spending habits).
- **Update Policy**: Synced daily via bank aggregator sync or manual logs.
- **Retrieval Strategy**: SQL ledger queries.
- **Expiration Policy**: Permanent.
- **Privacy Considerations**: High Sensitivity. Never sent to remote models. Any prompt needing financial analytics must receive masked/scrubbed ratios instead of raw amounts.

#### 13. Relationship Memory
- **Purpose**: Keeps track of contacts, social interactions, family events, and conversations.
- **Stored Information**: Contact profiles, birthdays, connection logs, and summaries of past meetups.
- **Relationships**: Linked to Calendar events.
- **Update Policy**: Updated on calendar event endings or manual relationship log notes.
- **Retrieval Strategy**: SQL lookup of upcoming birthdays, semantic search of past interactions.
- **Expiration Policy**: Permanent.
- **Privacy Considerations**: Moderate Sensitivity.

---

### Group C: Subjective Episodic & Working Memory

#### 14. Episodic Memory
- **Purpose**: Records temporal events, daily streams, diary-like narratives, and logs of "what happened on a given day".
- **Stored Information**: Daily Journal raw markdown files, stream transcripts, and timestamped journal entries.
- **Relationships**: Tied directly to Daily Calendar Events, Biometric Logs, and Daily Transaction lists.
- **Update Policy**: Appended nightly via the end-of-day daily review.
- **Retrieval Strategy**: Chronological sorting and vector proximity lookups.
- **Expiration Policy**: Indefinite.
- **Privacy Considerations**: High Sensitivity. Sent to remote services only if local context masking successfully sweeps the text.

#### 15. Semantic Memory
- **Purpose**: Extracts generalized core facts and concepts from the raw events of Episodic Memory.
- **Stored Information**: Extracted facts (e.g., "User experiences low energy on Tuesdays when sleep quality is below 60%"), synthesized preferences, and habits.
- **Relationships**: Formed by distilling Episodic Memory.
- **Update Policy**: Asynchronous background jobs executed during Weekly and Monthly Reviews.
- **Retrieval Strategy**: Priority semantic similarity matches.
- **Expiration Policy**: Indefinite.
- **Privacy Considerations**: High Sensitivity. Managed local-first.

#### 16. Working Memory
- **Purpose**: Represents the active, transient attention span of the AI session (the current context window).
- **Stored Information**: Active user prompt, recently injected context chunks, intermediate agent thoughts, and conversation history.
- **Relationships**: Feeds directly into the LLM context window.
- **Update Policy**: Modified in real-time as a chat session progresses.
- **Retrieval Strategy**: Loaded directly into memory threads.
- **Expiration Policy**: Purged immediately upon session termination.
- **Privacy Considerations**: Highly dynamic. Relies heavily on the `ContextMaskingService` to intercept payloads.

---

## 2. Ingestion & Retrieval Pipelines

### Memory Ingestion Pipeline
```mermaid
flowchart TD
    Raw[Raw Input: Note, Journal, Event, or Log] --> Filter[Context Masking: Local Scrubbing Engine]
    Filter --> SQL[Insert Relational Data into PostgreSQL]
    Filter --> Chunk[Semantic Chunker: 512 Char Windows]
    Chunk --> Embed[ONNX Local Embedder: generate 384-dim Vector]
    Embed --> Vector[Upsert into pgvector Store]
    SQL -.-> Graph[Update Node Relationship Cache in SQLite]
    Vector -.-> Graph
Memory Retrieval Pipeline & Hybrid Search
flowchart TD
    Query[User Query / Agent Request] --> EmbedQuery[ONNX: Generate Query Vector]
    EmbedQuery --> VecSearch[pgvector Cosine Search: <=> operator]
    Query --> FTSSearch[PostgreSQL Full-Text Search: GIN Index]
    VecSearch --> RRF[Hybrid Ranking Engine: Reciprocal Rank Fusion]
    FTSSearch --> RRF
    RRF --> Filter[Filter by Metadata: Tags, Recency, User Permission]
    Filter --> Ranking[Rank by Memory Score: Relevance, Importance, Recency]
    Ranking --> Compress[Context Compression: LLMLingua / LongLLMLingua]
    Compress --> Assembly[Assemble final Context Payload]
3. Context Generation, Ranking & Compression
To prevent the LLM context window from flooding with irrelevant or redundant data, Life OS implements a deterministic Ranking and Compression Engine.

Context Generation Flow
flowchart TD
    Prompt[User Prompt: e.g., 'Review my sleep habits'] --> Working[Working Memory Session]
    Working --> Engine[Memory Retrieval Pipeline]
    Engine --> Rel[Relational Context: Goals, Habits from SQL]
    Engine --> Sem[Semantic Context: Note embeddings from pgvector]
    Engine --> Epi[Episodic Context: Daily Journals]
    Rel --> Combine[Combine Context Chunks]
    Sem --> Combine
    Epi --> Combine
    Combine --> Rank[Score Context Chunks: Relevance x Recency]
    Rank --> Compress[Context Compression: Remove redundant tokens]
    Compress --> Assembly[Inject System Instructions & Identity]
    Assembly --> Final[Final Context Window sent to LLM]
Memory Ranking Algorithm (Memory Score)
Every retrieved memory chunk is evaluated and scored dynamically using the following formula:

$$\text{Memory Score} = (w_1 \cdot \text{Semantic Relevance}) + (w_2 \cdot \text{Temporal Recency}) + (w_3 \cdot \text{Static Importance})$$

Semantic Relevance: Cosine similarity between the Query Vector and the Chunk Vector (value between 0 and 1).

Temporal Recency: Decay function defined as:

$$\text{Recency} = e^{-\lambda \cdot t}$$

where $t$ represents time elapsed since creation (in days) and $\lambda$ is the decay parameter (default $\lambda = 0.05$).

Static Importance: Hardcoded or distilled weight of the memory type (e.g., Identity = 1.0, Values = 0.9, Goal = 0.8, Task completed = 0.2).

Context Compression Mechanics
Deduplication: Context chunks with cosine similarities $> 0.85$ are flagged, and only the chunk with the highest metadata priority is retained.
Token Pruning: Utilizes a lightweight local model (like LLMLingua) to calculate perplexity-based token importance. Redundant formatting, auxiliary verbs, and low-information tokens are stripped, reducing context payload size by up to 40% without losing semantic comprehension.
4. Knowledge Graph Integration
Life OS bridges vector storage and relational relational structures into a Semantic Graph. While pgvector retrieves conceptually similar chunks, the relational indexes in SQLite/PostgreSQL act as strict traversal paths.

Knowledge Graph Schema
classDiagram
    class UserNode {
        +UUID id
    }
    class GoalNode {
        +UUID id
        +String title
    }
    class ProjectNode {
        +UUID id
        +String name
    }
    class TaskNode {
        +UUID id
        +String title
    }
    class NoteNode {
        +UUID id
        +String title
    }
    class JournalNode {
        +UUID id
        +Date date
    }

    UserNode "1" -- "0..*" GoalNode : ESTABLISHES
    UserNode "1" -- "0..*" JournalNode : LOGS
    GoalNode "1" -- "0..*" ProjectNode : DRIVES
    ProjectNode "1" -- "0..*" TaskNode : DECOMPOSES
    ProjectNode "1" -- "0..*" NoteNode : REFERENCES
    JournalNode "1" -- "0..*" NoteNode : REFLECTS
    TaskNode "1" -- "0..*" NoteNode : CONNECTS
    NoteNode "1" -- "0..*" NoteNode : BACKLINKS_TO
By leveraging this integrated system, an AI Agent querying about a specific GoalNode can dynamically traverse the graph, immediately retrieving the exact linked ProjectNodes, active TaskNodes, and relevant NoteNodes via simple relational joins before applying semantic vector search on adjacent nodes. This guarantees exactness and context relevance.

