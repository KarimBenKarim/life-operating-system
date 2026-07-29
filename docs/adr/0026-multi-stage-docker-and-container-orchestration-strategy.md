# ADR 0026: Multi-Stage Docker and Container Orchestration Strategy

## Status
Accepted

## Context
When deploying the Life OS server-side components (API Gateway, WebSocket pub/sub daemons, Redis session caches, background indexing agents) to staging and production platforms, we require a repeatable, secure, and resource-efficient packaging model. Standard Docker builds compile and execute inside a single image, dragging unnecessary compilers, local SDK tools, and devDependencies into the final container. This results in bloated image payloads ($>1\text{GB}$) and increases the container's attack surface.

## Decision
We select **Multi-Stage Docker builds** combined with **Docker Compose / Swarm** container orchestration:
1. **Multi-Stage Builds**: Explicitly split into a `builder` phase (which loads compilation SDKs, devDependencies, and executes Vite/JS/Rust compilation) and a `runner` phase (which imports only production dependencies and compiled static assets).
2. **Container Isolation**: Running containers execute as non-privileged users (`USER lifeos`), blocking root container privilege escalations.
3. **Orchestration**: Docker Compose manages local development and self-hosted layouts, and Docker Swarm handles staging and production horizontal replication.

## Rationale
- **Minimal Image Size**: Multi-stage builds shrink image payloads down to $<150\text{MB}$, accelerating CI/CD container registry pushes and cluster rollouts.
- **Enhanced Security**: Eliminating compilers (like `g++` or `make`) and development tools from the production container prevents attackers from compiling local exploits if a web shell vulnerability is ever discovered.
- **Low Operational Overhead**: Docker Swarm provides simple horizontal autoscaling, rolling updates, and network segregation without the extreme orchestration complexity and memory footprints of standard Kubernetes.

## Consequences
- **Positive**: Extremely fast builds, highly secure isolated containers, lightweight memory footprints, and simple horizontal scaling.
- **Negative**: Requires writing more complex multi-stage Dockerfiles and managing separate compose templates for dev and prod.
