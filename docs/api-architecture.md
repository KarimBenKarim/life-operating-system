# Life OS API Architecture Specification

This document specifies the complete **API Architecture** for **Life OS**. It outlines a multi-protocol interface (REST, WebSockets, and Tauri IPC), defines security and gateway standards, drafts comprehensive REST endpoints for 11 functional domains, includes OpenAPI recommendations, and outlines three core sequence diagrams.

---

## 1. Multi-Protocol API Architecture

Life OS provides three primary protocol boundaries to balance standard transactional CRUD, real-time client state synchronizations, and resource-efficient local executions.

                      +-----------------------------------+
                      |       Life OS Frontend Client      |
                      +-----------------------------------+
                        /                |                \
                       /                 |                 \
              (HTTPS) /            (WSS) |                  \ (Tauri IPC)
                     v                   v                   v
        +-------------------+   +------------------+   +-------------------+
        |     REST API      |   |   WebSocket API  |   |   Local Tauri     |
        |     Gateway       |   |   Pub/Sub Engine |   |   IPC Command     |
        +-------------------+   +------------------+   +-------------------+

### Protocol Rationale
1. **REST Gateway (HTTPS)**: Default for structured transactional CRUD operations, authentication, user settings, data exports, and file-upload operations.
2. **WebSocket Pub/Sub Engine (WSS)**: Delivers duplex communication. Relays background file-watcher synchronization changes, live biometric sensor data stream pushes, and conversational AI agent message packets directly to UI dashboards.
3. **Local Tauri IPC Channel**: Direct Rust-to-JS native channel utilized exclusively inside local desktop binaries to bypass network latency and access physical device APIs directly.
4. **AI Service APIs (MCP / JSON-RPC)**: Implements the **Model Context Protocol (MCP)** using standard JSON-RPC 2.0 over STDIO or SSE, allowing LLMs to safely query databases and search vaults via structured function tools.

---

## 2. Core Gateway & Security Standards

### Authentication & Authorization
- **Authentication**: Stateless session management via **JSON Web Tokens (JWT)**.
  - JWT is passed inside the `Authorization: Bearer <TOKEN>` header.
  - Access Tokens expire in `15 minutes`. Refresh Tokens are stored as HTTP-only, secure, same-site cookies, expiring in `7 days`.
- **Authorization**: **Role-Based Access Control (RBAC)** + **Resource-level Ownership**.
  - Users can only mutate resources where `resource.user_id == current_user.id`.
  - Administrative endpoints (e.g., system backups, server logs) require `role == 'admin'`.

### Versioning, Pagination, and Filtering
- **Versioning**: Explicit URL path versioning (e.g., `/api/v1/`).
- **Pagination**: Zero-indexed limit-offset query parameters for flat tables:
  
  `GET /api/v1/tasks?limit=20&offset=40`
  
  Returns metadata payload containing `total_count`, `limit`, and `offset`.
- **Filtering**: Structured column filters passed as URL query parameters:
  
  `GET /api/v1/transactions?filter[category]=groceries&filter[status]=cleared`

### Error Handling & Rate Limiting
- **Standardized Error Format**: Adheres strictly to the RFC 7807 Problem Details specification.
  ```json
  {
    "type": "https://api.lifeos.org/errors/validation-failed",
    "title": "Validation Failed",
    "status": 422,
    "detail": "Due date cannot be in the past.",
    "instance": "/api/v1/tasks/task_90e3ab7d",
    "errors": [
      {
        "field": "due_date",
        "message": "Must be greater than or equal to 2026-07-27"
      }
    ]
  }
Rate Limiting:
Authenticated Users: 1200 requests per minute.
Unauthenticated Users (e.g., Sign-In, Sign-Up): 30 requests per minute (by client IP).
Emits standard X-RateLimit-* headers. Returns 429 Too Many Requests on exhaustion.
3. REST Endpoint Groups
A. Core Personal Domains
1. Users & Identity
POST /api/v1/auth/register - Registers a new user.
POST /api/v1/auth/login - Authenticates user; returns access token and sets HTTP-only secure refresh cookie.
POST /api/v1/auth/refresh - Requests a new access token via refresh token validation.
GET /api/v1/users/me - Retrieves active user profile.
PATCH /api/v1/users/me - Updates active user timezone, preferences, or password.
2. Goals
POST /api/v1/goals - Establishes a new long-term goal.
GET /api/v1/goals - Lists all active and archived user goals (paginated).
GET /api/v1/goals/{id} - Retrieves details of a specific goal.
PATCH /api/v1/goals/{id} - Updates goal milestones or values.
DELETE /api/v1/goals/{id} - Removes a goal (cascades or sets null on dependent projects).
3. Projects
POST /api/v1/projects - Deploys a new outcome-driven project.
GET /api/v1/projects - Queries active and backlog projects (with filtering).
GET /api/v1/projects/{id} - Details a project and lists its associated tasks.
PATCH /api/v1/projects/{id} - Updates project status, description, or target dates.
DELETE /api/v1/projects/{id} - Soft-deletes a project.
4. Tasks
POST /api/v1/tasks - Creates an actionable task item.
GET /api/v1/tasks - Queries the task backlog (filtering by priority, status, or project).
PATCH /api/v1/tasks/{id} - Modifies priority, state (completed, archived), or schedule dates.
DELETE /api/v1/tasks/{id} - Hard/soft deletes a task.
5. Memories (Journals)
POST /api/v1/journals - Commits a Daily, Weekly, or Monthly journal entry.
GET /api/v1/journals - Queries journal list (filtering by type, sentiment, or date range).
GET /api/v1/journals/{id} - Details raw markdown path, subjective mood ratings, and metrics.
PATCH /api/v1/journals/{id} - Updates mood rating, narrative, or sentiment scores.
6. Knowledge (Documents & Notes)
POST /api/v1/documents - Uploads/registers a new evergreen document or file asset.
GET /api/v1/documents - Lists notes, files, and links (supports full-text search query).
GET /api/v1/documents/{id} - Retrieves note text content or S3 attachment download URLs.
POST /api/v1/documents/{id}/links - Creates a directional backlink (graph edge) to another note.
B. System & AI Agent Domains
7. AI Agents
POST /api/v1/agents - Registers a specialized agent persona.
GET /api/v1/agents - Queries active agent list.
POST /api/v1/agents/{id}/chat - Dispatches a prompt to the agent, running context masking and retrieving local RAG context.
PATCH /api/v1/agents/{id}/instructions - Modifies agent persona settings or system constraints.
8. Reports
POST /api/v1/reports - Compiles a new telemetry report (e.g., "Health Correlation" or "Finance Summary").
GET /api/v1/reports - Lists compiled reports.
GET /api/v1/reports/{id} - Returns PDF download link or raw JSON statistical metrics.
9. Dashboards
GET /api/v1/dashboard/metrics - Aggregates real-time overview counts (Habit streaks, outstanding tasks, month-to-date spending ratio, mood scores).
10. Notifications
GET /api/v1/notifications - Lists pending, unread, or read user notifications.
POST /api/v1/notifications/{id}/approve - User signs off on a pending critical agentic tool execution (Human-in-the-Loop workflow).
PATCH /api/v1/notifications/mark-read - Sets is_read = true for active logs.
11. Administration
GET /api/v1/admin/audit-logs - System-wide audit trails (for system monitoring).
POST /api/v1/admin/backups - Forces snapshot creation of relational database and Note Vault.
4. Architectural Sequence Diagrams
Sequence A: User Authentication & Token Handshake
sequenceDiagram
    autonumber
    actor User as User Client
    participant GW as API Gateway
    participant DB as Postgres Relational DB
    participant Cache as Redis Store

    User->>GW: POST /api/v1/auth/login {email, password}
    GW->>DB: Query User record by email
    DB-->>GW: Return hash verification
    GW->>GW: Validate hash securely
    GW->>Cache: Save Refresh Session (uuid)
    GW-->>User: Return HTTP Access Token (JSON) & Secure Refresh Cookie
Sequence B: Relational Goal & Project Creation
sequenceDiagram
    autonumber
    actor User as User Client
    participant GW as API Gateway
    participant DB as Postgres Relational DB
    participant Sync as Real-time Event Router (WS)

    User->>GW: POST /api/v1/projects {name, goal_id, target_date}
    GW->>DB: Validate user authentication & goal_id ownership
    GW->>DB: INSERT INTO projects (name, goal_id, user_id)
    DB-->>GW: Return UUID & Transaction success
    GW->>Sync: Publish 'project.create' event to user socket
    Sync-->>User: Push real-time WS update to dashboard view
    GW-->>User: HTTP 201 Created (JSON Payload)
Sequence C: Real-time Event & Biometric Push
sequenceDiagram
    autonumber
    participant Daemon as Bank API Daemon (Cron)
    participant GW as API Gateway
    participant Cache as Redis Store
    participant WS as WebSocket Pub/Sub Server
    actor User as Connected User Client

    Daemon->>GW: Ingest new transaction 'Gym Membership -$45'
    GW->>Cache: Update dashboard metric caches
    GW->>WS: Broadcast 'live.finance.transaction' event
    WS->>User: Dispatch WSS frame (Live toast: 'New transaction logged!')
5. OpenAPI 3.0 Structural Recommendations
Life OS contracts must follow standard contract-first OpenAPI schemas. The core schema hierarchy is structured as follows:

openapi: 3.0.3
info:
  title: Life OS Core API
  version: 1.0.0
  description: Private, secure REST & WS Personal Knowledge Graph API Gateway.
servers:
  - url: https://api.lifeos.org/api/v1
    description: Production Server Gateway
paths:
  /auth/login:
    post:
      summary: User Login
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/LoginRequest'
      responses:
        '200':
          description: Successful Authentication
          headers:
            Set-Cookie:
              schema:
                type: string
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/LoginResponse'
        '401':
          $ref: '#/components/responses/UnauthorizedError'
components:
  schemas:
    LoginRequest:
      type: object
      required:
        - email
        - password
      properties:
        email:
          type: string
          format: email
        password:
          type: string
          format: password
    LoginResponse:
      type: object
      properties:
        access_token:
          type: string
        token_type:
          type: string
          example: Bearer
        expires_in:
          type: integer
          example: 900
  responses:
    UnauthorizedError:
      description: Authentication tokens missing or invalid.
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ProblemDetails'
    ProblemDetails:
      type: object
      properties:
        type:
          type: string
        title:
          type: string
        status:
          type: integer
        detail:
          type: string
