# Life OS Database & Persistence Architecture

This document specifies the complete persistence architecture for **Life OS**. It outlines a multi-tiered data storage strategy, designs a comprehensive relational PostgreSQL schema, defines vector memory pipelines, sets data lifecycle rules, establishes scalability standards, and includes detailed architectural diagrams.

---

## 1. Multi-Tiered Data Storage Strategy

Life OS uses a **hybrid multi-tiered storage engine** to satisfy transactional speed, permanent readability, low-latency search, and AI-native semantic memory retrieval.

              +-----------------------------------+
              |      User Actions & UI Layer      |
              +-----------------------------------+
                                |
                                v
              +-----------------------------------+
              |      Persistence Router API       |
              +-----------------------------------+
                 /              |              \
                /               |               \
               v                v                v
    +-------------------+ +-----------+ +-------------------+
    |  Local MD Vault   | |  Caching  | |   Relational DB   |
    |  (Raw Documents,  | |  (Redis)  | |  (PostgreSQL /    |
    |  Journals, Notes) | |           | |   TimescaleDB)    |
    +-------------------+ +-----------+ +-------------------+
             |                                   |
             | (Extract Embeddings)              | (Relational IDs)
             v                                   v
    +-------------------------------------------------------+
    |                  Vector Search DB                     |
    |                    (pgvector)                         |
    +-------------------------------------------------------+

### Storage Components Rationale

1. **Relational Database (PostgreSQL)**: Serves as the central transaction ledger. It maintains ACID-compliant relational states for users, tasks, accounts, transaction logs, habit performance logs, calendars, notifications, and telemetry. It enforces referential integrity across the personal knowledge workspace.
2. **Vector Memory Database (`pgvector` / SQLite-vec)**: Houses low-latency high-dimensional embeddings generated from raw documents, conversation histories, and note chunks to enable Retrieval-Augmented Generation (RAG) and semantic query capabilities.
3. **Object Storage (Local FS / S3)**: Optional S3-compatible object storage (or partitioned local directory folders) for attachments, biometric binary data exports (e.g., Apple Health raw XML zip), and document PDF uploads.
4. **Cache Strategy (Redis / Local In-Memory Cache)**: Redis handles session states, transient prompt anonymization maps, rate-limiting counters, and pre-computed dashboard statistics (e.g., weekly habit streaks and monthly net-worth aggregations).
5. **Search Indexing**: Implements multi-model search combining **PostgreSQL Full-Text Search (FTS)** with BM25 algorithms for keyword matching, and vector cosine similarity for conceptual context.

---

## 2. Relational Database Schema Design (PostgreSQL)

This schema represents the highly normalized PostgreSQL engine powering Life OS, featuring strict constraints, partial indexes, and optimized cascade behaviors.

```sql
-- Life OS SQL DDL Schema Specification
-- Normalisation: Third Normal Form (3NF)

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ==========================================
-- 1. Users & Identity Domain
-- ==========================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100),
    timezone VARCHAR(50) DEFAULT 'UTC' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE TABLE identity_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) DEFAULT 'local' NOT NULL, -- e.g., 'local', 'google', 'github'
    provider_user_id VARCHAR(255),
    metadata JSONB, -- stores flexible auth claims securely
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_user_provider UNIQUE(user_id, provider)
);

-- ==========================================
-- 2. Goals & Personal Orchestration Domain
-- ==========================================

CREATE TABLE goals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    target_date DATE,
    metric_name VARCHAR(100),
    metric_target NUMERIC(12, 2),
    metric_current NUMERIC(12, 2) DEFAULT 0.00 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE TYPE project_status AS ENUM ('backlog', 'active', 'completed', 'paused');

CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    goal_id UUID REFERENCES goals(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status project_status DEFAULT 'backlog' NOT NULL,
    target_date DATE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high', 'urgent');
CREATE TYPE task_status AS ENUM ('inbox', 'next', 'scheduled', 'completed', 'archived');

CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status task_status DEFAULT 'inbox' NOT NULL,
    priority task_priority DEFAULT 'medium' NOT NULL,
    due_date DATE,
    recurrence_rule VARCHAR(100), -- RFC 5545 iCalendar Recurrence format
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    CONSTRAINT check_due_date_sane CHECK (due_date IS NULL OR due_date >= '1970-01-01')
);

-- ==========================================
-- 3. Habits & Routines Domain
-- ==========================================

CREATE TYPE habit_interval AS ENUM ('daily', 'weekly', 'monthly');

CREATE TABLE habits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    cue VARCHAR(255),
    reward VARCHAR(255),
    interval habit_interval DEFAULT 'daily' NOT NULL,
    target_frequency INT DEFAULT 1 NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE TABLE habit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    habit_id UUID NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    completed_value NUMERIC(12, 2) DEFAULT 1.00 NOT NULL,
    logged_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- ==========================================
-- 4. Calendars & Scheduling Domain
-- ==========================================

CREATE TABLE calendars (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(10) DEFAULT '#3b82f6' NOT NULL,
    is_external BOOLEAN DEFAULT FALSE NOT NULL,
    external_sync_url VARCHAR(2048)
);

CREATE TABLE calendar_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    calendar_id UUID NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    is_all_day BOOLEAN DEFAULT FALSE NOT NULL,
    recurrence_rule VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    CONSTRAINT check_times CHECK (end_time >= start_time)
);

-- ==========================================
-- 5. Memories & Personal Journals Domain
-- ==========================================

CREATE TYPE journal_type AS ENUM ('daily', 'weekly', 'monthly', 'annual');

CREATE TABLE journals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type journal_type DEFAULT 'daily' NOT NULL,
    base_date DATE NOT NULL,
    raw_markdown_path VARCHAR(1024) NOT NULL, -- points to MD Vault location
    subjective_mood INT CHECK (subjective_mood BETWEEN 1 AND 10),
    sentiment_score NUMERIC(3, 2), -- ranges from -1.00 to +1.00
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_user_period_journal UNIQUE(user_id, type, base_date)
);

-- ==========================================
-- 6. Knowledge & Documents Domain
-- ==========================================

CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    raw_markdown_path VARCHAR(1024), -- if stored in Note Vault
    storage_object_key VARCHAR(1024), -- if stored in Blob S3
    file_size_bytes BIGINT,
    mime_type VARCHAR(100) DEFAULT 'text/markdown' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Graph associations (Backlinks and entity tags)
CREATE TABLE note_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_note_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    target_note_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) DEFAULT 'backlink' NOT NULL,
    CONSTRAINT check_non_self_referential CHECK (source_note_id != target_note_id)
);

-- ==========================================
-- 7. Intelligence & Agent Configuration Domain
-- ==========================================

CREATE TABLE ai_agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    persona_role VARCHAR(100) NOT NULL, -- e.g., 'financial_advisor'
    system_instructions TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- ==========================================
-- 8. Communication & Auditing Domains
-- ==========================================

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    is_read BOOLEAN DEFAULT FALSE NOT NULL,
    action_url VARCHAR(2048),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE TABLE system_audit_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action_type VARCHAR(100) NOT NULL, -- e.g., 'user_auth', 'goal_create'
    entity_name VARCHAR(100) NOT NULL, -- e.g., 'tasks', 'users'
    entity_id UUID,
    before_state JSONB,
    after_state JSONB,
    client_ip VARCHAR(45),
    logged_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- ==========================================
-- Indexing Optimization Strategy
-- ==========================================

-- Index for scanning active items quickly
CREATE INDEX idx_tasks_user_status ON tasks(user_id, status) WHERE status != 'completed' AND status != 'archived';
CREATE INDEX idx_projects_status ON projects(user_id, status);
-- High performance temporal joins
CREATE INDEX idx_habit_logs_timestamp ON habit_logs(habit_id, logged_at DESC);
CREATE INDEX idx_calendar_events_range ON calendar_events(calendar_id, start_time, end_time);
-- Search indexes
CREATE INDEX idx_documents_trgm ON documents USING gin (title gin_trgm_ops);
CREATE INDEX idx_system_audit_logs_user_action ON system_audit_logs(user_id, action_type);
3. Vector Memory Specification
Life OS acts as an interactive second brain. Text-based knowledge must be converted into high-dimensional space vector embeddings, facilitating semantically rich interactions with local agents.

Semantic Memory Flow Diagram
sequenceDiagram
    autonumber
    participant FS as MD Vault File System
    participant CR as Core Engine Daemon
    participant HF as Local Embedding Engine (ONNX)
    participant VS as Vector Database (pgvector)

    FS->>CR: File Modified / Created (e.g., Note or Journal)
    CR->>CR: Extract Document Content & Strip Sensitivity
    CR->>CR: Run Semantic Chunker
    Note over CR: Recursively split into 500-character chunks (with 10% overlap)
    CR->>HF: Pass cleaned text chunks
    HF->>HF: Generate 384-dim Vectors (All-MiniLM-L6-v2)
    HF-->>CR: Return high-dimensional floats
    CR->>VS: Upsert Vectors with Metadata (Doc ID, Title, Tags, Paragraph)
    VS-->>User: Indexed and Ready for AI Agent Queries
Chunking Strategy & Embedding Lifecycle
Chunking algorithm: Employs an recursive character text splitter. Splits at markdown section markers (##), double newlines (\n\n), single newlines, and spaces.
Optimal size: 512 characters.
Overlap: 64 characters (to preserve semantic transitions).
Embedding Model: Local default is all-MiniLM-L6-v2 (384-dimensions, extremely fast inference on standard CPUs, packaged as an ONNX runtime binary).
Metadata Payload:
document_id: Source relation identifier linking back to PostgreSQL documents table.
title: Title of the note for contextual prompt injection.
header_context: Hierarchy of header path (e.g., "Health > Sleep Patterns").
tags: Indexed frontmatter arrays for strict namespace filtering.
Retrieval Process: Uses Hybrid Search. Combines standard PostgreSQL Full Text Search (FTS) score with pgvector cosine distance score (<=> operator) weighted dynamically based on user query length.
4. Data Lifecycle Management
To maintain system integrity and prevent disk bloating over a multi-decade scale, the engine applies automated lifecycle policies.

Step	Action details	Triggers / Policies
Creation	Schema structures are generated automatically via SQL migrations. All relational records use native UUIDv4 tags.	On-demand user action or automated ingestion API.
Versioning	Standard notes maintain history files under a local hidden .vault/history/ directory to prevent DB table bloat.	Triggered on note saving with character difference > 5%.
Updates	Atomic transactions in PostgreSQL. If update fails, SQLite/Markdown sync handles immediate rollbacks.	Synchronous API or asynchronous file watchers.
Archiving	Tasks and Projects are transitioned to status 'archived'. Archiving indexes removes them from default fast scans while retaining relational history.	Manual user sweep or goals completed > 90 days ago.
Retention	Notification and Audit Log tables auto-purge records older than 365 days to local gzipped backup files on external storage.	Cron daemon execution at 02:00 daily.
Deletion	Hard delete is avoided for user-facing domain roots (Soft-delete via deleted_at field). Hard delete triggers on system logs if requested.	Cascade deletes configured cleanly for dependent entities (e.g., deleting a habit cascade deletes habit logs).
5. Architectural Diagrams
Entity-Relationship Diagram (ERD)
This physical ERD shows the cross-domain relational linkages within Life OS.

erDiagram
    USERS ||--o{ IDENTITY_PROFILES : has
    USERS ||--o{ GOALS : establishes
    USERS ||--o{ PROJECTS : manages
    USERS ||--o{ TASKS : schedules
    USERS ||--o{ HABITS : commits
    USERS ||--o{ CALENDARS : owns
    USERS ||--o{ JOURNALS : records
    USERS ||--o{ DOCUMENTS : writes
    USERS ||--o{ AI_AGENTS : configures
    USERS ||--o{ NOTIFICATIONS : receives

    GOALS ||--o{ PROJECTS : drives
    PROJECTS ||--o{ TASKS : decomposes

    HABITS ||--o{ HABIT_LOGS : collects
    CALENDARS ||--o{ CALENDAR_EVENTS : schedules

    DOCUMENTS ||--o{ NOTE_RELATIONSHIPS : connects_source
    DOCUMENTS ||--o{ NOTE_RELATIONSHIPS : connects_target
Storage Architecture
graph TD
    Client[Life OS Client App] --> API[Core Persistence Router]
    
    subgraph Local Storage
        API --> MD[Markdown File Engine]
        MD --> Vault[Physical MD Note Vault]
        API --> SqlCache[(Local SQLite Cache)]
    end

    subgraph Server Storage
        API --> PG[(Central PostgreSQL DB)]
        PG --> PGV[pgvector Vector DB]
        API --> Redis[(Redis Cache)]
    end

    subgraph Object Store
        API --> S3[(S3 Blob Attachments)]
    end
6. Scalability, Backup, and Disaster Recovery
Scalability Strategy
Database Partitioning: system_audit_logs and habit_logs are partitioned by range (Monthly) using PostgreSQL native partitioning to guarantee write performance as logs accumulate.
Read Replication: Supports a primary-replica architecture. Analytical queries (e.g., generating year-over-year sleep or spend trends) execute against a read-replica node, insulating the primary write ledger from lag.
Connection Pooling: Uses PgBouncer to manage high-frequency stateless connections from background agent daemons.
Backup & Disaster Recovery (DR)
Continuous Archiving: Employs WAL-G or pgBackRest for automated, encrypted point-in-time recovery (PITR) backups streamed directly to private S3-compatible cloud storage.
RTO & RPO:
Recovery Time Objective (RTO): < 1 Hour.
Recovery Point Objective (RPO): < 10 Minutes (WAL logs streamed continuously).
The Ultimate Fallback (Sovereignty Tenet): Because the raw Markdown note files stand as the core source of truth, a full server disaster can be recovered locally simply by pointing a new Life OS client app to the directory of Markdown files, triggering an automated SQLite index rebuild.
