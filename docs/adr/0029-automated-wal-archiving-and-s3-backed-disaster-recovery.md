# ADR 0029: Automated WAL Archiving and S3-Backed Disaster Recovery

## Status
Accepted

## Context
At a multi-decade scale, a personal operating system holds the entirety of a user's life telemetry (biometrics, finances, journals, knowledge base). Ensuring absolute data durability is a primary system requirement. If a physical device or virtual machine experiences sudden hardware failure or storage corruption, we must guarantee that the database can be reconstructed with minimal data loss.

## Decision
We select **WAL-G / pgBackRest** for continuous relational PostgreSQL database replication and **S3-backed zero-knowledge object storage** for encrypted disaster recovery:
1. **Continuous WAL Streaming**: Write-Ahead Logs (WAL) are streamed continuously from the production database to private, user-owned S3-compatible zero-knowledge cloud buckets.
2. **Point-In-Time Recovery (PITR)**: Full encrypted physical base backups are compiled weekly. Combined with continuous WAL files, users can restore their database state to any specific minute in history.
3. **The Sovereignty Tenet**: If a complete cloud provider disaster occurs, recovery remains instantaneous. Since the user's local Markdown Note Vault stands as the core source of truth, users can rebuild their database relational indexes from scratch simply by mounting their Markdown directory to a fresh client install.

## Rationale
- **Minimal RPO**: Continuous WAL streaming guarantees a Recovery Point Objective (RPO) of $< 10\text{ minutes}$, keeping potential data loss to a bare minimum.
- **Zero-Knowledge Security**: Backups are fully encrypted client-side using password-derived AES keys before being uploaded to S3, ensuring cloud storage providers cannot inspect personal files.
- **Lifetime Sovereignty**: Decoupling the database relational cache from the plain-text markdown source of truth guarantees that no database corruption can ever cause permanent, irreversible loss of the user's notes and journals.

## Consequences
- **Positive**: Exceptional database durability, sub-10 minute data loss windows, point-in-time state recovery, and complete zero-knowledge physical backup ownership.
- **Negative**: Requires configuring S3 integration keys and incurs minor cloud object storage costs.
