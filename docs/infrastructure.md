# Life OS Infrastructure & Deployment Architecture Specification

This document specifies the complete **Infrastructure Architecture** for **Life OS**. It outlines local development configurations, server-side Docker architectures, multi-stage container structures, native Tauri packaging, CI/CD automation pipelines, telemetry (Prometheus, Loki, OpenTelemetry), backup/DR configurations, and multi-stage deployment environments.

---

## 1. Environment & Multi-Stage Deployment Strategy

Life OS uses a **hybrid, multi-stage deployment model**.

```
+-----------------------------------------------------------------------------------------------+
| Stage             | Core Engine         | Data Relational Cache   | AI Inference Engine       |
+-------------------+---------------------+-------------------------+---------------------------+
| 1. Local          | Local Tauri Client  | Embedded SQLite         | Local Ollama (Llama 3)    |
| 2. Development    | Docker Container    | Development Postgres    | Remote Ollama / Cloud LLM |
| 3. Staging        | Docker Swarm Container| Staging PostgreSQL DB  | Dedicated Cloud LLM Gateway|
| 4. Production     | Docker Swarm Cluster| High-Availability RDS   | Dual Local + Secure LLM   |
+-----------------------------------------------------------------------------------------------+
```

### Stage Recommendations & Environments

#### 1. Local (Sovereignty Default)
- **Primary runtime**: Tauri application running natively on user's workstation.
- **Relational Cache**: SQLite encrypted locally via SQLCipher.
- **AI Processing**: Local **Ollama** running `llama3:8b` or `mistral:7b` over local CPU/GPU hardware.
- **Note Storage**: Direct reads/writes to local file system directories.

#### 2. Development (Dev Sandbox)
- **Primary runtime**: Microservices compiled locally and run inside **Docker Compose** containers.
- **Relational Cache**: Shared PostgreSQL developer container on local network.
- **AI Processing**: Shared local Ollama daemon or public sandbox API endpoints (OpenAI / Anthropic) with simulated context masking.
- **Telemetry**: Localized Prometheus/Grafana instance scraping dev containers.

#### 3. Staging (Pre-Release Testing)
- **Primary runtime**: Pre-release Docker images deployed to a local home server or private cloud virtual machine via **Docker Swarm**.
- **Relational Cache**: Single-node dedicated PostgreSQL instance with automated daily backup snapshots.
- **AI Processing**: Secure, private LLM API endpoints with strict logging and context masking enabled.
- **Telemetry**: Full Prometheus, Loki, and Jaeger tracing stack active to monitor multi-agent latency profiles.

#### 4. Production (Enterprise Scale / High Availability)
- **Primary runtime**: Redundant microservice containers orchestrated across a secure private server cluster (using Docker Swarm or lightweight K3s).
- **Relational Cache**: High-availability PostgreSQL primary-replica cluster with automated continuous WAL-G stream backups.
- **AI Processing**: Localized dedicated embedding models (ONNX) combined with fine-tuned local models or secure cloud gateways using hardware keychains.
- **Telemetry**: Production-grade central telemetry dashboards with automated pager alerts (via Prometheus Alertmanager).

---

## 2. Container Strategy & Docker Architecture

The server-side API Gateway, real-time WebSocket pub/sub daemon, and background indexing agents are fully containerized using **optimized multi-stage Dockerfiles** to isolate environment dependencies, speed up builds, and maintain tiny execution payloads ($<150MB$ production image sizes).

### 1. Multi-Stage Dockerfile Specification (Backend Core API)
```dockerfile
# ==========================================
# Stage 1: Build & Compilation Environment
# ==========================================
FROM node:20-alpine AS builder

# Install system compilation dependencies
RUN apk add --no-cache libc6-compat python3 make g++

WORKDIR /usr/src/app

# Cache package dependencies
COPY package*.json ./
RUN npm ci

# Copy source code and build production assets
COPY . .
RUN npm run build

# ==========================================
# Stage 2: Production Runtime Environment
# ==========================================
FROM node:20-alpine AS runner

ENV NODE_ENV=production
WORKDIR /usr/share/lifeos

# Create non-privileged system user for process isolation
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 lifeos

# Install production dependencies only
COPY package*.json ./
RUN npm ci --only=production

# Copy compiled build artifacts from stage 1
COPY --from=builder /usr/src/app/dist ./dist

# Apply file permissions to secure running workspace
RUN chown -R lifeos:nodejs /usr/share/lifeos
USER lifeos

EXPOSE 8080
ENV PORT=8080

CMD ["node", "dist/main.js"]
```

### 2. Multi-Container Orchestration (Docker Compose Configuration)
For development and lightweight self-hosting deployments, services are coordinated via a unified `docker-compose.yml` manifest, implementing strict memory limits and network partition blocks:

```yaml
version: '3.8'

networks:
  lifeos-internal:
    driver: bridge
    internal: true # Isolates DB and Cache from public internet
  lifeos-public:
    driver: bridge

services:
  gateway:
    image: lifeos/api-gateway:latest
    ports:
      - "8080:8080"
    environment:
      - NODE_ENV=production
      - REDIS_URL=redis://cache:6379/0
      - DATABASE_URL=postgresql://life_admin:SecurePass123!@database:5432/lifeos_db
    depends_on:
      - database
      - cache
    networks:
      - lifeos-public
      - lifeos-internal
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M

  websocket-server:
    image: lifeos/websocket-pubsub:latest
    ports:
      - "8081:8081"
    environment:
      - REDIS_URL=redis://cache:6379/1
    depends_on:
      - cache
    networks:
      - lifeos-public
      - lifeos-internal
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 256M

  database:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=life_admin
      - POSTGRES_PASSWORD=SecurePass123!
      - POSTGRES_DB=lifeos_db
    volumes:
      - pg-data:/var/lib/postgresql/data
    networks:
      - lifeos-internal
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G

  cache:
    image: redis:7-alpine
    command: redis-server --appendonly yes # Enforces persistence
    volumes:
      - redis-data:/data
    networks:
      - lifeos-internal
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 256M

volumes:
  pg-data:
  redis-data:
```

---

## 3. Network Architecture & Security Topology

The Life OS server deployment implements a strict **defense-in-depth network topology**. All database, caching, and background compute containers are completely air-gapped inside a secure private internal network.

```mermaid
graph TD
    User([External Client / Web Client]) -->|HTTPS:8080 / TLS 1.3| Nginx[Nginx Reverse Proxy / Cloudflare]
    Nginx -->|Proxy Pass| Gateway[API Gateway Container]
    Nginx -->|WSS:8081 / TLS 1.3| WS[WebSocket Pub/Sub Container]

    subgraph Secure Private Internal Network [Trust Zone 2 & 3: Air-gapped Network]
        Gateway -->|Port:5432| DB[(PostgreSQL Database Cluster)]
        Gateway -->|Port:6379| Cache[(Redis Session Cache)]
        WS -->|Port:6379| Cache
        Agent[Asynchronous AI Agent Worker] -->|Port:5432| DB
        Agent -->|Port:6379| Cache
    end
```

### Network Safeguards
- **Inbound Filtering**: The reverse proxy (Nginx / Caddy) enforces HSTS, strict rate limits, and rejects all HTTP requests that do not present a valid Host Header.
- **Isolated Database Port**: Database port `5432` and Redis port `6379` are not exposed to the host machine's interface, blocking direct SSH/port manipulation from outside.

---

## 4. CI/CD Automation Pipeline

Life OS employs a fully automated, contract-first **GitHub Actions CI/CD Pipeline**.

```mermaid
graph TD
    Push[Code Push / PR to main] --> Lint[Lint & Formatting: ESLint / Rustfmt]
    Lint --> Test[Unit & Contract Tests: Jest / Cargo Test]
    Test --> Security[Static SAST Scan: cargo-audit / npm audit]

    Security --> Split{Build Target Check}

    Split -->|Tauri Client| TauriBuild[Tauri Cross-Platform Compilation]
    TauriBuild --> Release[Draft GitHub Release with Installers]

    Split -->|Docker Server| DockerBuild[Docker Multi-Stage Build & Push]
    DockerBuild --> Deploy[Deploy Cluster: Docker Swarm Rollout]
```

### Pipeline Workflow Description
1. **Quality & Validation**: Every commit triggers syntax linting and structural unit tests.
2. **Contract Consistency**: OpenAPI schemas are verified using validator scripts to confirm complete type alignment before builds.
3. **Security Auditing**: The pipeline runs SAST static scanning (such as `trivy` for container CVEs and `cargo-audit` for dependency audits) to detect vulnerabilities.
4. **Client Compilation (Tauri Builder)**: Native runners (macOS, Windows, Ubuntu VM blocks) build native release binaries concurrently, signing packages with cryptographic developer keys and drafting a release payload.
5. **Server Compilation (Docker Builder)**: Server-side images are compiled, tagged using Semantic Versioning (`vX.Y.Z`) and pushed to a secure, private registry (GitHub Container Registry).
6. **Continuous Rollout (CD)**: Executes an automated rolling update inside production clusters, checking for healthy container handshakes before de-allocating old containers.

---

## 5. CENTRALIZED TELEMETRY, LOGGING & DISTRIBUTED TRACING

To manage operational health and monitor complex multi-agent reasoning paths (where asynchronous agents invoke tools or pass events to one another), the gateway orchestrates a **centralized observability stack**.

```mermaid
graph TD
    App[Containers / Tauri Core] -->|OpenTelemetry spans| Jaeger[(Jaeger Tracing Engine)]
    App -->|JSON Logs via Promtail| Loki[(Grafana Loki Log Hub)]
    App -->|Metrics via scrape_configs| Prom[(Prometheus Metrics Collector)]

    Loki --> Grafana[Grafana Unified Visualization Dashboard]
    Prom --> Grafana
    Jaeger --> Grafana
```

### Observability Stack Components

#### 1. Centralized Logging (Grafana Loki & Promtail)
- **Format**: All application containers output standard JSON logs to `stdout`/`stderr`.
- **Scraping**: Promtail gathers logs, appends metadata labels (e.g., `service_name="api-gateway"`, `container_id="c1fcf895"`), and streams them directly to Loki.
- **Payload Schema**:
  ```json
  {
    "timestamp": "2026-07-27T10:14:00Z",
    "level": "INFO",
    "service": "api-gateway",
    "user_id": "u1b2c3d4",
    "message": "User login completed successfully.",
    "context": { "ip": "192.168.1.12" }
  }
  ```

#### 2. Metrics Collection (Prometheus)
- **Scraping**: Prometheus polls metrics endpoints (`/metrics`) on a **15-second interval**.
- **Metrics Collected**:
  - `http_requests_total`: Monotonically increasing counter, partitioned by status code and method.
  - `active_websocket_connections_total`: Gauge tracking concurrent live streams.
  - `agent_mcp_tool_execution_duration_seconds`: Histogram measuring execution speed of agent tools.
  - `database_pool_active_connections`: Gauge tracking SQLite/Postgres connection pooling.

#### 3. Distributed Tracing (OpenTelemetry & Jaeger)
- **The Problem**: Asynchronous agents pass JSON-RPC messages internally over a Local Event Bus. Standard logging cannot easily reconstruct how an agent made a final decision.
- **The Solution**: Every agent interaction and tool execution is wrapped inside an **OpenTelemetry Trace Span**.
- **Trace Propagation**: When an action starts, the system creates a `trace_id`. This ID is injected into the JSON-RPC message header:
  `"context_id": "trace_881a29cf-a0e2"`
  Every subsequent sub-agent and tool call appends a child `span_id`. Jaeger stitches these spans together, presenting a beautiful horizontal waterfall visualization of the complete cognitive decision tree.

---

## 6. Config, Secrets & Disaster Recovery (DR)

### 1. Configuration Management
- **Environment Separation**: Application behavior is controlled using standard `.env` configuration files.
- **Local vs Cloud**: Local installations ignore Cloud properties (e.g., `PLAID_CLIENT_ID` or `AWS_S3_BUCKET`), disabling remote routes.
- **Production Schema validation**: On application start, the config service runs JSON-schema checks. If mandatory fields (e.g., `DATABASE_URL`) are missing or misformatted, the service forcefully aborts start.

### 2. Scalability Strategy
- **Horizontal Container Replication**: API Gateways and background agent containers scale horizontally inside the production cluster using automated resource autoscalers.
- **Connection Pooling**: Utilizes **PgBouncer** to pool database connections, ensuring high-frequency agent queries do not exhaust database limits.
- **Read Replicas**: High-load read requests (such as historical sleep and financial dashboards) are routed to dedicated Postgres read-replicas, isolating the write-heavy database transactions.

### 3. Backup Strategy & Disaster Recovery
- **Continuous Archive (WAL-G)**: Relational PostgreSQL databases stream Write-Ahead Logs (WAL) continuously to private, zero-knowledge S3 object storage (via `WAL-G` or `pgBackRest`), offering **Point-in-Time Recovery (PITR)**.
- **Disaster Recovery Targets (RTO / RPO)**:
  - **Recovery Time Objective (RTO)**: $< 1\text{ Hour}$ (Time elapsed from a system crash until full operational state recovery).
  - **Recovery Point Objective (RPO)**: $< 10\text{ Minutes}$ (Maximum permissible data loss window, governed by WAL streaming intervals).
- **The Ultimate Fallback (Sovereignty Default)**: If a total cloud provider disaster occurs, recovery is instantaneous. Since the user's raw Markdown note vault is the source of truth, the user can spin up a new local Tauri client, mount their Markdown note directory, and trigger an automated SQLite relational database rebuild, recovering their entire life log offline.
