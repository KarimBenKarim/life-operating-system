# Life OS Domain Model Specification

This document provides a complete, formal Domain-Driven Design (DDD) domain model for **Life OS**. It outlines bounded contexts, aggregate boundaries, entities, value objects, domain services, ownership, lifecycle rules, invariants, and class diagrams.

---

## 1. Bounded Contexts Map

To prevent a "Big Ball of Mud" and separate concerns across the personal knowledge workspace, Life OS is divided into **six Bounded Contexts**.

+---------------------------------------------------------------------------------+ | LIFE OS GRAPH | +---------------------------------------------------------------------------------+ | | |
v v v
+-------------------+ +-------------------+ +-------------------+
| Personal | | Habit Formation | | Self-Reflection |
| Orchestration | | & Behavior | | & Knowledge |
| Context | | Context | | Context |
+-------------------+ +-------------------+ +-------------------+
| | |
v v v
+-------------------+ +-------------------+ +-------------------+
| Financial Ledger | | Biometrics | | Agentic Assistant |
| Context | | & Vitality | | Context |
| | | Context | | (MCP & RAG) |
+-------------------+ +-------------------+ +-------------------+


### Context Responsibilities

1. **Personal Orchestration Context (Tasks & Projects)**: Responsible for planning, prioritizing, and executing outcome-driven tasks.
2. **Habit Formation & Behavior Context (Habits & Streaks)**: Tracks repetitive daily behaviors, routines, cues, rewards, and behavioral statistics.
3. **Self-Reflection & Knowledge Context (Journals & Notes)**: Manages periodic journaling (Daily, Weekly, Monthly) and non-temporal notes, bookmarks, and wiki documents.
4. **Financial Ledger Context (Accounts & Ledgers)**: Maintains double-entry transactions, accounts (assets, liabilities, credit), and expense/income categorization.
5. **Biometrics & Vitality Context (Sleep & Health)**: Tracks health metrics (sleep latency, duration, activity logs, steps, subjective mood scores, water intake).
6. **Agentic Assistant Context (AI Agents, MCP, & RAG)**: Handles semantic document embedding, Model Context Protocol (MCP) tool routing, context masking/scrubbing, and agent prompt generation.

---

## 2. Core Domain Entities & Value Objects

### Domain Taxonomy (Entity, Value Object, Aggregate Root)

- **Aggregate Root (AR)**: A cluster of domain objects that can be treated as a single unit for data changes. It maintains transactional boundaries and relational consistency.
- **Entity (E)**: An object defined by its unique identity rather than its attributes. It has a continuous lifecycle thread.
- **Value Object (VO)**: An immutable object defined entirely by its attributes. It has no conceptual identity and is used to describe characteristics.

---

## 3. Context Specifications

### A. Personal Orchestration Context

Handles actionable items and long-term milestones.

```mermaid
classDiagram
    direction LR
    class Project {
        <<Aggregate Root>>
        +UUID id
        +String name
        +String description
        +ProjectStatus status
        +Date targetDate
        +create()
        +archive()
    }
    class Task {
        <<Aggregate Root>>
        +UUID id
        +UUID projectId
        +String title
        +TaskStatus status
        +Priority priority
        +Date dueDate
        +RecurrenceRule recurrence
        +complete()
        +reschedule()
    }
    class TaskStatus {
        <<enumeration>>
        INBOX
        NEXT
        SCHEDULED
        COMPLETED
        ARCHIVED
    }
    class Priority {
        <<enumeration>>
        LOW
        MEDIUM
        HIGH
        URGENT
    }
    class RecurrenceRule {
        <<Value Object>>
        +String frequency
        +Integer interval
        +List~String~ daysOfWeek
    }

    Project "1" *-- "0..*" Task : owns
    Task o-- TaskStatus : has
    Task o-- Priority : has
    Task *-- RecurrenceRule : configures
Invariants & Lifecycles
Invariants:
A task's completed timestamp cannot be prior to its creation timestamp.
A project's status cannot be marked COMPLETED if it owns tasks that are still in NEXT or SCHEDULED status.
Ownership:
Project acts as an Aggregate Root. It can exist without tasks.
Task can exist independently (representing single inbox captures) or stand as a child owned by a Project.
Lifecycle:
Tasks begin as INBOX. They can progress through NEXT or SCHEDULED before transitioning to COMPLETED or ARCHIVED.
B. Habit Formation & Behavior Context
Fosters positive behavior loops through cues and rewards.

classDiagram
    direction LR
    class Habit {
        <<Aggregate Root>>
        +UUID id
        +String name
        +HabitTarget target
        +String cue
        +String reward
        +Boolean isActive
        +logPerformance()
    }
    class HabitLog {
        <<Entity>>
        +UUID id
        +UUID habitId
        +Float completedValue
        +DateTime timestamp
    }
    class HabitTarget {
        <<Value Object>>
        +Integer frequencyPerInterval
        +IntervalType interval
        +Float targetQuantity
        +String metricUnit
    }
    class IntervalType {
        <<enumeration>>
        DAILY
        WEEKLY
        MONTHLY
    }

    Habit "1" *-- "0..*" HabitLog : records
    Habit *-- HabitTarget : targets
    HabitTarget o-- IntervalType : interval
Invariants & Lifecycles
Invariants:
A HabitLog must reference a valid and active parent Habit.
The performance value logged cannot be negative.
Ownership:
Habit is the Aggregate Root.
HabitLog is an Entity whose lifecycle is strictly bound to its parent Habit. If a Habit is deleted, all associated HabitLog objects are cascaded.
C. Self-Reflection & Knowledge Context
Preserves subjective journals, thoughts, and objective knowledge assets.

classDiagram
    direction LR
    class Journal {
        <<Aggregate Root>>
        +UUID id
        +RecurrenceType type
        +Date baseDate
        +String rawMarkdown
        +Sentiment sentiment
        +saveDraft()
    }
    class Note {
        <<Aggregate Root>>
        +UUID id
        +String title
        +String content
        +Frontmatter metadata
        +List~Link~ backlinks
        +updateContent()
    }
    class Sentiment {
        <<Value Object>>
        +Integer moodScore
        +String narrativeSummary
    }
    class RecurrenceType {
        <<enumeration>>
        DAILY
        WEEKLY
        MONTHLY
        ANNUAL
    }
    class Frontmatter {
        <<Value Object>>
        +Map~String-String~ tags
        +DateTime lastModified
    }

    Journal *-- Sentiment : records
    Journal o-- RecurrenceType : type
    Note *-- Frontmatter : metadata
Invariants & Lifecycles
Invariants:
A Journal is unique to its baseDate and RecurrenceType. For example, there can only exist one Daily Journal for 2026-07-27.
A Note title must not be empty.
Ownership:
Both Journal and Note stand alone as distinct Aggregate Roots.
Links represent weak associations (edges) in the database graph cache rather than tight cascade-delete boundaries.
D. Financial Ledger Context
Provides atomic double-entry tracking of wealth.

classDiagram
    direction LR
    class Account {
        <<Aggregate Root>>
        +UUID id
        +String name
        +AccountType type
        +Money currentBalance
        +adjustBalance()
    }
    class Transaction {
        <<Aggregate Root>>
        +UUID id
        +UUID sourceAccountId
        +UUID destinationAccountId
        +Money amount
        +String payee
        +String category
        +Date transactionDate
        +TransactionStatus status
        +reconcile()
    }
    class Money {
        <<Value Object>>
        +Float amount
        +String currency
    }
    class AccountType {
        <<enumeration>>
        ASSET
        LIABILITY
        CREDIT_CARD
        INVESTMENT
    }
    class TransactionStatus {
        <<enumeration>>
        PENDING
        CLEARED
        VOID
    }

    Account *-- Money : currentBalance
    Account o-- AccountType : type
    Transaction *-- Money : amount
    Transaction o-- TransactionStatus : status
Invariants & Lifecycles
Invariants:
Every transaction must state non-zero values.
An Account's computed balance must always reflect the aggregate value of all of its reconciled/cleared transactions.
Ownership:
Account and Transaction are distinct Aggregate Roots.
Transactions refer to accounts via IDs (sourceAccountId, destinationAccountId), maintaining strong referential boundaries.
E. Biometrics & Vitality Context
Systematically tracks body performance, energy level, and sleep quality.

classDiagram
    direction LR
    class MetricLog {
        <<Aggregate Root>>
        +UUID id
        +Date trackingDate
        +SleepData sleep
        +ActivityMetrics activity
        +Integer moodRating
        +updateLogs()
    }
    class SleepData {
        <<Value Object>>
        +Integer durationMinutes
        +Integer qualityPct
        +Integer deepSleepMinutes
    }
    class ActivityMetrics {
        <<Value Object>>
        +Integer stepsCount
        +Integer activeCaloriesBurned
        +Integer waterIntakeMl
    }

    MetricLog *-- SleepData : sleep
    MetricLog *-- ActivityMetrics : activity
Invariants & Lifecycles
Invariants:
A MetricLog is unique per date.
Percentage metrics (e.g., sleep.qualityPct) must reside strictly between 0 and 100.
Ownership:
MetricLog serves as the Aggregate Root.
SleepData and ActivityMetrics are immutable Value Objects fully managed under the MetricLog boundary.
F. Agentic Assistant Context
Exposes the user's graph securely to AI engines while keeping personal facts private.

classDiagram
    direction LR
    class PromptSession {
        <<Aggregate Root>>
        +UUID id
        +String rawUserPrompt
        +String maskedPrompt
        +AnonymizationMap tokenMap
        +createMaskedPayload()
        +restoreResponse()
    }
    class AnonymizationMap {
        <<Value Object>>
        +Map~String-String~ mappings
    }
    class MCPTool {
        <<Entity>>
        +String name
        +String description
        +String schemaJson
    }

    PromptSession *-- AnonymizationMap : tokenMap
Invariants & Lifecycles
Invariants:
Any raw sensitive metadata (e.g., bank accounts or exact salary logs) must never be present inside the maskedPrompt field.
Ownership:
PromptSession is a transient Aggregate Root. It is created on prompt generation, acts as a filter, and is deleted or archived once the response has been returned and contextualized to the user.
4. Domain Services
While entities represent state and ID, some domain processes are inherently stateless and span multiple aggregates.

1. ContextMaskingService
Responsibility: Scans incoming text, queries local SQLite for sensitivity filters, replaces highly sensitive context (e.g., bank account strings, names) with abstract tokens, and reverses the mapping upon receiving LLM outputs.
Bounded Context: Agentic Assistant Context.
2. PersonalKnowledgeGraphBuilder
Responsibility: Listens to changes in the local Markdown note files. Extracts frontmatter, cross-links, and hashtags, then performs transactional updates in SQLite to keep the relational and semantic graph index completely synced with the physical file system.
Bounded Context: Self-Reflection & Knowledge Context.
3. BiometricPatternDetector
Responsibility: Runs multi-variable regression analysis locally on MetricLog records and HabitLog tracks to output personal correlations (e.g., "Sleeping less than 350 minutes correlates to a 20% mood drop and high sugar-craving habit loops the following day").
Bounded Context: Biometrics & Vitality Context.
5. Architectural Assumptions & Modeling Decisions
Hybrid Plain-text Markdown + SQLite Architecture
Decision: All notes and journals are saved as local Markdown files. All tasks, transactions, and biometric records are mirrored in a relational SQLite DB.
Justification: Storing documents in markdown guarantees complete durability. SQLite is chosen because querying graphs and computing metrics (streaks, financial ledger sums, tasks pending) is slow and intensive if processed by parsing hundreds of text files dynamically. SQLite mirrors the state to ensure sub-millisecond response rates for local dashboards.
Relational Boundaries over Aggregate Roots
Decision: Bounded contexts do not share write transactions. Instead, they reference each other via UUIDs (e.g., Task refers to Project via projectId).
Justification: Using IDs ensures that aggregates can be easily partitioned, synchronized separately across devices, and tested in isolation.
Transient AI Prompt Sessions
Decision: AI prompt masking sessions do not persist permanently on disk.
Justification: Keeping transient logs prevents accidental local accumulation of prompt session databases that could expose sensitive context or increase local vulnerability.
